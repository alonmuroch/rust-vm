use core::{cmp, marker::PhantomData, ptr};

use crate::global::{PAGE_ALLOC, ROOT_PPN};
use crate::BootInfo;
use types::{
    Sv32PagePerms, Sv32PageTable, SV32_DIRECT_MAP_BASE, SV32_PAGE_SIZE, SV32_VPN_MASK, map_allocating,
    SV32_PTE_R, SV32_PTE_W, SV32_PTE_X, SV32_PTE_V,
};

const PAGE_SIZE: usize = SV32_PAGE_SIZE;
const DIRECT_MAP_BASE: usize = SV32_DIRECT_MAP_BASE as usize;

/// Permissions used by the kernel/user mapping helpers.
pub type PagePerms = Sv32PagePerms;

#[derive(Debug, Clone, Copy)]
pub struct PageAllocator {
    next_ppn: u32,
    limit_ppn: u32,
}

impl PageAllocator {
    /// Create a bump-frame allocator over [start_ppn, limit_ppn).
    pub const fn new(start_ppn: u32, limit_ppn: u32) -> Self {
        Self {
            next_ppn: start_ppn,
            limit_ppn,
        }
    }

    /// Allocate the next free physical page number, or None if exhausted.
    pub fn alloc(&mut self) -> Option<u32> {
        if self.next_ppn >= self.limit_ppn {
            return None;
        }
        let ppn = self.next_ppn;
        self.next_ppn += 1;
        Some(ppn)
    }

    /// Zero a 4 KiB page in guest physical memory via the direct map.
    fn zero_page(ppn: u32) {
        let base = (ppn as usize)
            .checked_mul(PAGE_SIZE)
            .expect("page offset overflow");
        let virt = direct_map_addr(base).expect("direct map overflow while zeroing page");
        unsafe {
            ptr::write_bytes(virt as *mut u8, 0, PAGE_SIZE);
        }
    }
}

/// Return the kernel's current root PPN (satp PPN field).
pub fn current_root() -> u32 {
    unsafe { *ROOT_PPN.get_mut() }
}

/// Initialize the kernel MMU allocator state from bootloader handoff.
pub fn init(boot_info: &BootInfo) {
    unsafe {
        *ROOT_PPN.get_mut() = boot_info.root_ppn;
        let limit_ppn = (boot_info.memory_size as usize / PAGE_SIZE) as u32;
        *PAGE_ALLOC.get_mut() = Some(PageAllocator::new(boot_info.next_free_ppn, limit_ppn));
    }
}

/// Allocate and zero a fresh L1 root page table. Returns None if out of frames.
pub fn alloc_root() -> Option<u32> {
    let alloc = unsafe { PAGE_ALLOC.get_mut() };
    match alloc {
        Some(alloc) => {
            let root = alloc.alloc()?;
            PageAllocator::zero_page(root);
            Some(root)
        }
        None => None,
    }
}

/// Map a user-visible virtual range with the provided permissions into a specific root.
pub fn map_user_range_for_root(root_ppn: u32, va_start: u32, len: usize, perms: PagePerms) -> bool {
    let alloc = unsafe { PAGE_ALLOC.get_mut() };
    match alloc {
        Some(alloc) => {
            let mapper = KernelMapper::new(alloc);
            map_allocating(&mapper, root_ppn, va_start, len, perms)
        }
        None => false,
    }
}

/// Map a user-visible virtual range with the provided permissions into the current root.
pub fn map_user_range(va_start: u32, len: usize, perms: PagePerms) -> bool {
    let root = unsafe { *ROOT_PPN.get_mut() };
    map_user_range_for_root(root, va_start, len, perms)
}

/// Walk Sv32 to translate a VA in the given root to a physical address.
pub fn translate_user_va(root_ppn: u32, va: u32) -> Option<usize> {
    let vpn1 = (va >> 22) & SV32_VPN_MASK;
    let vpn0 = (va >> 12) & SV32_VPN_MASK;
    let offset = (va & 0xfff) as usize;

    let l1_base = (root_ppn as usize)
        .checked_mul(PAGE_SIZE)?;
    let l1_addr = l1_base + vpn1 as usize * core::mem::size_of::<u32>();
    let l1_pte = read_pte(l1_addr)?;
    if l1_pte & SV32_PTE_V == 0 || l1_pte & (SV32_PTE_R | SV32_PTE_W | SV32_PTE_X) != 0 {
        return None;
    }

    let l2_base = ((l1_pte >> 10) as usize)
        .checked_mul(PAGE_SIZE)?;
    let l2_addr = l2_base + vpn0 as usize * core::mem::size_of::<u32>();
    let l2_pte = read_pte(l2_addr)?;
    if l2_pte & SV32_PTE_V == 0 {
        return None;
    }

    let ppn = (l2_pte >> 10) as usize;
    ppn.checked_mul(PAGE_SIZE)?.checked_add(offset)
}

/// Copy data into a user VA range for a specific root using the direct-map window.
pub fn copy_into_user(root_ppn: u32, va_start: u32, data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    let mut remaining = data.len();
    let mut src_off = 0usize;
    let mut va = va_start;
    while remaining > 0 {
        let phys = match translate_user_va(root_ppn, va) {
            Some(p) => p,
            None => return false,
        };
        let page_off = (va as usize) & (PAGE_SIZE - 1);
        let to_copy = cmp::min(remaining, PAGE_SIZE - page_off);
        let dst = match direct_map_addr(phys) {
            Some(v) => v,
            None => return false,
        };
        unsafe {
            ptr::copy_nonoverlapping(
                data.as_ptr().add(src_off),
                dst as *mut u8,
                to_copy,
            );
        }
        remaining -= to_copy;
        src_off += to_copy;
        va = va.wrapping_add(to_copy as u32);
    }
    true
}

/// Sv32 page-table accessor that routes PTE traffic through the kernel's direct map.
struct KernelMapper<'a> {
    alloc: *mut PageAllocator,
    _marker: PhantomData<&'a mut PageAllocator>,
}

impl<'a> KernelMapper<'a> {
    fn new(alloc: &'a mut PageAllocator) -> Self {
        Self {
            alloc: alloc as *mut PageAllocator,
            _marker: PhantomData,
        }
    }
}

impl<'a> Sv32PageTable for KernelMapper<'a> {
    fn page_size(&self) -> usize {
        PAGE_SIZE
    }

    fn read_pte(&self, phys_addr: usize) -> Option<u32> {
        let va = direct_map_addr(phys_addr)?;
        Some(unsafe { (va as *const u32).read_volatile() })
    }

    fn write_pte(&self, phys_addr: usize, val: u32) {
        if let Some(va) = direct_map_addr(phys_addr) {
            unsafe { (va as *mut u32).write_volatile(val) };
        }
    }

    fn alloc_frame(&self) -> Option<u32> {
        let alloc = unsafe { &mut *self.alloc };
        alloc.alloc()
    }

    fn zero_frame(&self, ppn: u32) {
        PageAllocator::zero_page(ppn);
    }
}

fn direct_map_addr(phys: usize) -> Option<usize> {
    DIRECT_MAP_BASE.checked_add(phys)
}

fn read_pte(phys_addr: usize) -> Option<u32> {
    let va = direct_map_addr(phys_addr)?;
    Some(unsafe { (va as *const u32).read_volatile() })
}
