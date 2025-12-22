use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;

use vm::memory::{Mmu, Perms, VirtualAddress, HEAP_PTR_OFFSET, PAGE_SHIFT};
use vm::metering::{MeterResult, Metering, MemoryAccessKind};

use crate::memory::pte::Pte;

/// Software Sv32 MMU backed by a contiguous physical buffer.
///
/// Design at a glance:
/// - Physical memory is a single `Vec<u8>` (`backing`). Frames are 4 KiB slices into it.
/// - Virtual→physical is resolved with Sv32-style page tables: L1 root (VPN1) and L2 (VPN0).
/// - Page tables themselves live in host memory (`root` + `l2_tables`) and store simple PTEs(page table entry).
/// - A bump frame allocator hands out PPNs (physical page numbers) sequentially from the backing; no free list yet.
/// - Mapping APIs (`map_page`/`map_range`) allocate tables/frames and set R/W/X/U bits.
/// - `translate` walks VPN1→VPN0, checks permissions against the access kind, and returns a byte
///   offset into the backing. All loads/stores go through this path.
/// - A guest heap bump pointer (`next_heap`) drives simple allocations; writes copy into backing
///   via translation to respect page boundaries.
///
/// Limitations/assumptions:
/// - No unmap or reuse of frames yet; the allocator only grows.
/// - No access/dirty bits; permissions are R/W/X/U/V only.
/// - `mem_slice` only returns contiguous slices when the mapped physical pages are contiguous.
/// - Identity mapping is not assumed; everything uses page tables even for kernel.
#[derive(Debug)]
pub struct Memory {
    /// Page size in bytes (Sv32: 4 KiB).
    page_size: usize,
    /// Total number of physical frames available.
    total_pages: usize,
    /// Contiguous physical backing store.
    backing: Rc<RefCell<Vec<u8>>>,
    /// Collection of root L1 tables (VPN1 index), one per address space.
    root_tables: RefCell<Vec<Box<[Pte; 1024]>>>,
    /// Index of the active root in `root_tables`.
    current_root: Cell<usize>,
    /// Pool of L2 tables (VPN0 index).
    l2_tables: RefCell<Vec<Box<[Pte; 1024]>>>,
    /// Bump allocator for guest heap pointer.
    next_heap: Cell<VirtualAddress>,
    /// Next free physical frame index for frame allocation.
    next_free_frame: Cell<usize>,
}

impl Memory {
    pub fn new(total_size_bytes: usize, page_size: usize) -> Self {
        assert!(page_size != 0, "page_size must be > 0");
        assert!(total_size_bytes != 0, "total_size_bytes must be > 0");

        let total_pages = (total_size_bytes + (page_size - 1)) / page_size;
        let total = total_pages
            .checked_mul(page_size)
            .expect("physical memory size overflow");
        Self {
            page_size,
            total_pages,
            backing: Rc::new(RefCell::new(vec![0u8; total])),
            root_tables: RefCell::new(vec![Box::new([Pte::default(); 1024])] ),
            current_root: Cell::new(0),
            l2_tables: RefCell::new(Vec::new()),
            next_heap: Cell::new(VirtualAddress(0)),
            next_free_frame: Cell::new(0),
        }
    }

    fn total_size(&self) -> usize {
        self.backing.borrow().len()
    }

    /// Allocate a fresh L1 root table and return its index.
    pub fn allocate_root(&self) -> usize {
        let mut roots = self.root_tables.borrow_mut();
        roots.push(Box::new([Pte::default(); 1024]));
        roots.len() - 1
    }

    /// Allocate a physical frame (4 KiB) and return its page number, or None if out of frames.
    fn allocate_frame(&self) -> Option<usize> {
        let frame = self.next_free_frame.get();
        if frame >= self.total_pages {
            return None;
        }
        self.next_free_frame.set(frame + 1);
        Some(frame)
    }

    /// Allocate a fresh L2 table and return its index in the L2 pool.
    fn allocate_l2(&self) -> usize {
        let mut l2s = self.l2_tables.borrow_mut();
        l2s.push(Box::new([Pte::default(); 1024]));
        l2s.len() - 1
    }

    /// Map a single 4 KiB page at `va` with the given permissions, allocating tables/frames as needed.
    fn map_page(&self, va: VirtualAddress, perms: Perms) {
        let vpn1 = va.vpn1() as usize;
        let vpn0 = va.vpn0() as usize;
        let mut roots = self.root_tables.borrow_mut();
        let current = self.current_root.get();
        let root = roots
            .get_mut(current)
            .unwrap_or_else(|| panic!("invalid root index {}", current));
        if root[vpn1].next_l2.is_none() {
            let l2_idx = self.allocate_l2();
            root[vpn1].next_l2 = Some(l2_idx);
            root[vpn1].valid = true;
        }
        let l2_idx = root[vpn1].next_l2.expect("l2 table missing");
        drop(roots);

        let mut l2s = self.l2_tables.borrow_mut();
        let l2 = &mut l2s[l2_idx];
        if !l2[vpn0].valid {
            let frame = self
                .allocate_frame()
                .expect("out of physical frames while mapping");
            l2[vpn0].ppn = frame;
            l2[vpn0].valid = true;
            l2[vpn0].read = perms.read;
            l2[vpn0].write = perms.write;
            l2[vpn0].exec = perms.exec;
            l2[vpn0].user = perms.user;
        }
    }

    /// Map a contiguous virtual range page-by-page with the given permissions.
    pub fn map_range(&self, start: VirtualAddress, len: usize, perms: Perms) {
        let mut page_start = start.align_down();
        let mut remaining = len;
        while remaining > 0 {
            self.map_page(page_start, perms);
            page_start =
                VirtualAddress(page_start.as_u32().wrapping_add(self.page_size as u32));
            if remaining > self.page_size {
                remaining -= self.page_size;
            } else {
                remaining = 0;
            }
        }
    }

    /// Translate a virtual address to a physical offset into `backing`, checking permissions.
    fn translate(&self, va: VirtualAddress, kind: MemoryAccessKind) -> Option<usize> {
        let vpn1 = va.vpn1() as usize;
        let vpn0 = va.vpn0() as usize;
        let offset = va.offset() as usize;
        let roots = self.root_tables.borrow();
        let root = roots
            .get(self.current_root.get())
            .unwrap_or_else(|| panic!("invalid root index {}", self.current_root.get()));
        let l2_idx = root.get(vpn1).and_then(|pte| pte.next_l2)?;
        let l2s = self.l2_tables.borrow();
        let l2 = l2s.get(l2_idx)?;
        let leaf = l2.get(vpn0)?;
        if !leaf.valid || !leaf.is_leaf() {
            return None;
        }
        // Basic permission check: align MemoryAccessKind to R/W.
        let allowed = match kind {
            MemoryAccessKind::Load | MemoryAccessKind::ReservationLoad => leaf.read || leaf.exec,
            MemoryAccessKind::Store
            | MemoryAccessKind::Atomic
            | MemoryAccessKind::ReservationStore => leaf.write,
        };
        if !allowed {
            return None;
        }
        let pa = (leaf.ppn << PAGE_SHIFT) + offset;
        Some(pa)
    }

    fn meter_access(
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
        addr: VirtualAddress,
        bytes: usize,
    ) -> bool {
        matches!(
            metering.on_memory_access(kind, addr.as_usize(), bytes),
            MeterResult::Continue
        )
    }

    /// Copy a slice into physical backing, honoring translation and page boundaries.
    fn copy_into_backing(&self, start: VirtualAddress, data: &[u8], kind: MemoryAccessKind) {
        let mut remaining = data.len();
        let mut offset_in_data = 0usize;
        let mut va = start;
        while remaining > 0 {
            let phys = self
                .translate(va, kind)
                .expect("copy failed: unmapped virtual address");
            let page_remaining = self.page_size - (va.offset() as usize);
            let to_copy = core::cmp::min(page_remaining, remaining);
            {
                let mut backing = self.backing.borrow_mut();
                let dst = phys;
                let src_start = offset_in_data;
                let src_end = src_start + to_copy;
                backing[dst..dst + to_copy].copy_from_slice(&data[src_start..src_end]);
            }
            remaining -= to_copy;
            offset_in_data += to_copy;
            va = VirtualAddress(va.as_u32().wrapping_add(to_copy as u32));
        }
    }

    /// Write bytes to an already mapped virtual region without advancing the heap.
    /// Callers must ensure the range is mapped and writable.
    pub fn write_bytes(&self, start: VirtualAddress, data: &[u8]) {
        self.copy_into_backing(start, data, MemoryAccessKind::Store);
    }
}

impl Mmu for Memory {
    fn mem(&self) -> Ref<Vec<u8>> {
        self.backing.borrow()
    }

    fn map_range(&self, start: VirtualAddress, len: usize, perms: Perms) {
        Memory::map_range(self, start, len, perms);
    }

    fn set_root(&self, root: usize) {
        let roots = self.root_tables.borrow();
        if root >= roots.len() {
            panic!("set_root: invalid root index {}", root);
        }
        self.current_root.set(root);
    }

    fn current_root(&self) -> usize {
        self.current_root.get()
    }

    fn mem_slice(
        &self,
        start: VirtualAddress,
        end: VirtualAddress,
    ) -> Option<std::cell::Ref<[u8]>> {
        if start.as_usize() > end.as_usize() {
            return None;
        }
        let len = end.as_usize().saturating_sub(start.as_usize());
        let phys_start = self.translate(start, MemoryAccessKind::Load)?;
        let last_va = VirtualAddress(end.as_u32().saturating_sub(1));
        let phys_last = self.translate(last_va, MemoryAccessKind::Load)?;
        // Ensure the range is physically contiguous.
        if phys_last + 1 != phys_start + len {
            return None;
        }
        let backing = self.backing.borrow();
        Some(std::cell::Ref::map(backing, move |v| {
            &v[phys_start..phys_start + len]
        }))
    }

    fn store_u16(
        &self,
        addr: VirtualAddress,
        val: u16,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 2) {
            return false;
        }
        if let Some(offset) = self.translate(addr, kind) {
            let mut backing = self.backing.borrow_mut();
            backing[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
        } else {
            return false;
        }
        true
    }

    fn store_u32(
        &self,
        addr: VirtualAddress,
        val: u32,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 4) {
            return false;
        }
        if let Some(offset) = self.translate(addr, kind) {
            let mut backing = self.backing.borrow_mut();
            backing[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        } else {
            return false;
        }
        true
    }

    fn store_u8(
        &self,
        addr: VirtualAddress,
        val: u8,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 1) {
            return false;
        }
        if let Some(offset) = self.translate(addr, kind) {
            let mut backing = self.backing.borrow_mut();
            backing[offset] = val;
        } else {
            return false;
        }
        true
    }

    fn load_u32(
        &self,
        addr: VirtualAddress,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        if !Self::meter_access(metering, kind, addr, 4) {
            return None;
        }
        let backing = self.backing.borrow();
        let offset = self.translate(addr, kind)?;
        Some(u32::from_le_bytes(
            backing[offset..offset + 4].try_into().unwrap(),
        ))
    }

    fn load_byte(
        &self,
        addr: VirtualAddress,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u8> {
        if !Self::meter_access(metering, kind, addr, 1) {
            return None;
        }
        let backing = self.backing.borrow();
        let offset = self.translate(addr, kind)?;
        Some(backing[offset])
    }

    fn load_halfword(
        &self,
        addr: VirtualAddress,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u16> {
        if !Self::meter_access(metering, kind, addr, 2) {
            return None;
        }
        let backing = self.backing.borrow();
        let offset = self.translate(addr, kind)?;
        Some(u16::from_le_bytes(
            backing[offset..offset + 2].try_into().unwrap(),
        ))
    }

    fn load_word(
        &self,
        addr: VirtualAddress,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        if !Self::meter_access(metering, kind, addr, 4) {
            return None;
        }
        let backing = self.backing.borrow();
        let offset = self.translate(addr, kind)?;
        Some(u32::from_le_bytes(
            backing[offset..offset + 4].try_into().unwrap(),
        ))
    }

    fn alloc_on_heap(&self, data: &[u8]) -> VirtualAddress {
        let mut addr = self.next_heap.get().as_u32();
        let align = 8;
        addr = (addr + (align - 1)) & !(align - 1);
        let end = addr + data.len() as u32;
        let start_va = VirtualAddress(addr);
        self.map_range(start_va, data.len(), Perms::rw_kernel());

        self.copy_into_backing(start_va, data, MemoryAccessKind::Store);
        let end_va = VirtualAddress(end);
        self.next_heap.set(end_va);
        start_va
    }

    fn stack_top(&self) -> VirtualAddress {
        VirtualAddress(self.total_size() as u32)
    }

    fn size(&self) -> usize {
        self.total_size()
    }

    fn offset(&self, addr: VirtualAddress) -> usize {
        addr.as_usize()
    }

    fn next_heap(&self) -> VirtualAddress {
        self.next_heap.get()
    }

    fn set_next_heap(&self, next: VirtualAddress) {
        self.next_heap.set(next);
    }
}
