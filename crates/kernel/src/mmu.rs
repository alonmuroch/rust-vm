use crate::global::Global;
use crate::BootInfo;

const PAGE_SIZE: usize = 4096;
const VPN_MASK: u32 = 0x3ff;
const PTE_V: u32 = 1 << 0;
const PTE_R: u32 = 1 << 1;
const PTE_W: u32 = 1 << 2;
const PTE_X: u32 = 1 << 3;
const PTE_U: u32 = 1 << 4;

#[derive(Clone, Copy)]
pub struct PagePerms {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
    pub user: bool,
}

impl PagePerms {
    pub const fn user_rwx() -> Self {
        Self {
            read: true,
            write: true,
            exec: true,
            user: true,
        }
    }
}

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

    /// Zero a 4 KiB page in guest physical memory.
    fn zero_page(ppn: u32) {
        let base = (ppn as usize) * PAGE_SIZE;
        unsafe {
            core::ptr::write_bytes(base as *mut u8, 0, PAGE_SIZE);
        }
    }
}

static ROOT_PPN: Global<u32> = Global::new(0);
static PAGE_ALLOC: Global<Option<PageAllocator>> = Global::new(None);

/// Initialize the kernel MMU allocator state from bootloader handoff.
pub fn init(boot_info: &BootInfo) {
    unsafe {
        *ROOT_PPN.get_mut() = boot_info.root_ppn;
        let limit_ppn = (boot_info.memory_size as usize / PAGE_SIZE) as u32;
        *PAGE_ALLOC.get_mut() = Some(PageAllocator::new(boot_info.next_free_ppn, limit_ppn));
    }
}

/// Map a user-visible virtual range with the provided permissions into the current root.
pub fn map_user_range(va_start: u32, len: usize, perms: PagePerms) -> bool {
    let root = unsafe { *ROOT_PPN.get_mut() };
    let alloc = unsafe { PAGE_ALLOC.get_mut() };
    match alloc {
        Some(alloc) => map_range(root, va_start, len, perms, alloc),
        None => false,
    }
}

fn map_range(root_ppn: u32, va_start: u32, len: usize, perms: PagePerms, alloc: &mut PageAllocator) -> bool {
    if len == 0 {
        return true;
    }
    let start = align_down(va_start as usize, PAGE_SIZE) as u32;
    let end = (va_start as usize).saturating_add(len);
    let end_aligned = align_up(end, PAGE_SIZE) as u32;

    let mut va = start;
    while va < end_aligned {
        if !map_page(root_ppn, va, perms, alloc) {
            return false;
        }
        va = va.wrapping_add(PAGE_SIZE as u32);
    }
    true
}

/// Map a single 4 KiB page at `va` into the two-level Sv32 page tables rooted at `root_ppn`.
/// Allocates an L2 table if the L1 entry is absent; refuses superpages; fills in a leaf PTE
/// with the requested permissions after zeroing the backing frame.
fn map_page(root_ppn: u32, va: u32, perms: PagePerms, alloc: &mut PageAllocator) -> bool {
    let vpn1 = (va >> 22) & VPN_MASK;
    let vpn0 = (va >> 12) & VPN_MASK;

    // L1 lookup
    let root_base = (root_ppn as usize) * PAGE_SIZE;
    let l1_entry_addr = root_base + vpn1 as usize * core::mem::size_of::<u32>();
    let mut l1_pte = unsafe { (l1_entry_addr as *const u32).read_volatile() };

    if l1_pte & PTE_V == 0 {
        let l2 = match alloc.alloc() {
            Some(ppn) => ppn,
            None => return false,
        };
        PageAllocator::zero_page(l2);
        l1_pte = ((l2 as u32) << 10) | PTE_V;
        unsafe { (l1_entry_addr as *mut u32).write_volatile(l1_pte) };
    } else if l1_pte & (PTE_R | PTE_W | PTE_X) != 0 {
        // Superpages not supported.
        return false;
    }

    let l2_ppn = l1_pte >> 10;
    let l2_base = (l2_ppn as usize) * PAGE_SIZE;
    let l2_entry_addr = l2_base + vpn0 as usize * core::mem::size_of::<u32>();

    let existing = unsafe { (l2_entry_addr as *const u32).read_volatile() };
    if existing & PTE_V != 0 {
        // Already mapped.
        return true;
    }

    let frame = match alloc.alloc() {
        Some(ppn) => ppn,
        None => return false,
    };
    PageAllocator::zero_page(frame);

    let mut flags = PTE_V;
    if perms.read {
        flags |= PTE_R;
    }
    if perms.write {
        flags |= PTE_W;
    }
    if perms.exec {
        flags |= PTE_X;
    }
    if perms.user {
        flags |= PTE_U;
    }
    let leaf = ((frame as u32) << 10) | flags;
    unsafe { (l2_entry_addr as *mut u32).write_volatile(leaf) };
    true
}

const fn align_up(val: usize, align: usize) -> usize {
    (val + (align - 1)) & !(align - 1)
}

const fn align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}
