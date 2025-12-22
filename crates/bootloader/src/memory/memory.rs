use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use vm::memory::{Mmu, Perms, VirtualAddress};
use vm::metering::{MeterResult, Metering, MemoryAccessKind};

// Minimal Sv32 bit layout for guest-visible PTEs.
const PTE_V: u32 = 1 << 0;
const PTE_R: u32 = 1 << 1;
const PTE_W: u32 = 1 << 2;
const PTE_X: u32 = 1 << 3;
const PTE_U: u32 = 1 << 4;

// Sv32 satp PPN mask (bits 21:0). Mode bits are ignored in this emulator.
const SATP_PPN_MASK: u32 = 0x003f_ffff;

/// Software Sv32 MMU backed by a contiguous physical buffer.
///
/// Design at a glance:
/// - Physical memory is a single `Vec<u8>` (`backing`). Frames are 4 KiB slices into it.
/// - Virtual→physical is resolved with Sv32-style page tables: L1 root (VPN1) and L2 (VPN0).
/// - Page tables live in guest memory; `translate` walks them using the satp root PPN.
/// - A bump frame allocator hands out PPNs (physical page numbers) sequentially from the backing; no free list yet.
/// - Mapping APIs (`map_page`/`map_range`) allocate tables/frames and set R/W/X/U bits.
/// - `translate` walks VPN1→VPN0, checks permissions against the access kind, and returns a byte
///   offset into the backing. All loads/stores go through this path.
/// - A guest heap bump pointer is tracked per-root.
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
    /// satp value that selects the active root PPN.
    satp: Cell<u32>,
    /// Per-root heap bump pointer (VA).
    per_root_heap: RefCell<HashMap<u32, VirtualAddress>>,
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
        // Reserve frame 0 for the initial root page table.
        let root_ppn: usize = 0;
        Self {
            page_size,
            total_pages,
            backing: Rc::new(RefCell::new(vec![0u8; total])),
            satp: Cell::new(root_ppn as u32),
            per_root_heap: RefCell::new(HashMap::new()),
            next_free_frame: Cell::new(root_ppn + 1),
        }
    }

    fn total_size(&self) -> usize {
        self.backing.borrow().len()
    }

    fn root_ppn(&self) -> usize {
        (self.satp.get() & SATP_PPN_MASK) as usize
    }

    fn root_base(&self) -> Option<usize> {
        let base = self.root_ppn().checked_mul(self.page_size)?;
        if base + self.page_size > self.total_size() {
            None
        } else {
            Some(base)
        }
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

    pub fn next_free_ppn(&self) -> usize {
        self.next_free_frame.get()
    }

    fn zero_frame(&self, ppn: usize) {
        let mut backing = self.backing.borrow_mut();
        let start = ppn
            .checked_mul(self.page_size)
            .expect("frame offset overflow");
        let end = start + self.page_size;
        backing[start..end].fill(0);
    }

    fn read_pte(&self, phys_addr: usize) -> Option<u32> {
        let backing = self.backing.borrow();
        let end = phys_addr.checked_add(4)?;
        if end > backing.len() {
            return None;
        }
        Some(u32::from_le_bytes(
            backing[phys_addr..end].try_into().unwrap(),
        ))
    }

    fn write_pte(&self, phys_addr: usize, val: u32) {
        let mut backing = self.backing.borrow_mut();
        let end = phys_addr
            .checked_add(4)
            .expect("pte write offset overflow");
        if end > backing.len() {
            panic!("pte write out of bounds");
        }
        backing[phys_addr..end].copy_from_slice(&val.to_le_bytes());
    }

    /// Map a single 4 KiB page at `va` with the given permissions, allocating tables/frames as needed.
    fn map_page(&self, va: VirtualAddress, perms: Perms) {
        let root_base = self.root_base().expect("invalid root page table base");
        let vpn1 = va.vpn1() as usize;
        let vpn0 = va.vpn0() as usize;

        // Ensure L2 table exists.
        let root_pte_addr = root_base + vpn1 * core::mem::size_of::<u32>();
        let root_pte = self.read_pte(root_pte_addr).unwrap_or(0);
        let root_is_valid = root_pte & PTE_V != 0;
        let root_has_perms = root_pte & (PTE_R | PTE_W | PTE_X) != 0;

        if root_is_valid && root_has_perms {
            panic!("superpages not supported");
        }

        let l2_ppn = if root_is_valid {
            (root_pte >> 10) as usize
        } else {
            let l2_frame = self
                .allocate_frame()
                .expect("out of physical frames while mapping L2");
            self.zero_frame(l2_frame);
            let new_pte = ((l2_frame as u32) << 10) | PTE_V;
            self.write_pte(root_pte_addr, new_pte);
            l2_frame
        };

        let l2_base = l2_ppn
            .checked_mul(self.page_size)
            .expect("l2 base overflow");
        let leaf_addr = l2_base + vpn0 * core::mem::size_of::<u32>();

        if let Some(existing) = self.read_pte(leaf_addr) {
            if existing & PTE_V != 0 {
                return;
            }
        }

        let frame = self
            .allocate_frame()
            .expect("out of physical frames while mapping leaf");
        self.zero_frame(frame);

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
        let leaf_pte = ((frame as u32) << 10) | flags;
        self.write_pte(leaf_addr, leaf_pte);
    }

    /// Map a contiguous virtual range page-by-page with the given permissions.
    pub fn map_range(&self, start: VirtualAddress, len: usize, perms: Perms) {
        if len == 0 {
            return;
        }
        let start_addr = start.align_down().as_usize();
        let end_addr = start.as_usize().saturating_add(len);

        let mut page_start = start_addr;
        while page_start < end_addr {
            self.map_page(VirtualAddress(page_start as u32), perms);
            page_start = page_start.saturating_add(self.page_size);
        }
    }

    /// Translate a virtual address to a physical offset into `backing`, checking permissions.
    ///
    /// This emulates an Sv32 page-table walk driven by the current `satp`:
    /// - `satp` PPN selects the root L1 page table (written by the kernel in guest memory).
    /// - We read the L1 PTE at VPN1; it must be valid and non-leaf (no superpages here).
    /// - From that PPN we read the L2 PTE at VPN0; it must be valid and carry R/W/X bits.
    /// - Permissions are checked against the access kind; on success we return a byte offset
    ///   into the physical backing buffer.
    ///
    /// All PTE bytes we read here are what the kernel previously wrote into guest memory;
    /// the host MMU just interprets them to enforce translations.
    fn translate(&self, va: VirtualAddress, kind: MemoryAccessKind) -> Option<usize> {
        let root_base = self.root_base()?;
        let vpn1 = va.vpn1() as usize;
        let vpn0 = va.vpn0() as usize;
        let offset = va.offset() as usize;

        let root_pte_addr = root_base + vpn1 * core::mem::size_of::<u32>();
        let root_pte = self.read_pte(root_pte_addr)?;
        if root_pte & PTE_V == 0 {
            return None;
        }

        // We only support two-level translation; reject L1 leaf/superpages.
        if root_pte & (PTE_R | PTE_W | PTE_X) != 0 {
            return None;
        }

        let l2_ppn = (root_pte >> 10) as usize;
        let l2_base = l2_ppn
            .checked_mul(self.page_size)
            .expect("l2 base overflow");
        let l2_pte_addr = l2_base + vpn0 * core::mem::size_of::<u32>();
        let l2_pte = self.read_pte(l2_pte_addr)?;
        if l2_pte & PTE_V == 0 {
            return None;
        }

        let allowed = match kind {
            MemoryAccessKind::Load | MemoryAccessKind::ReservationLoad => l2_pte & (PTE_R | PTE_X) != 0,
            MemoryAccessKind::Store | MemoryAccessKind::Atomic | MemoryAccessKind::ReservationStore => l2_pte & PTE_W != 0,
        };
        if !allowed {
            return None;
        }

        let leaf_ppn = (l2_pte >> 10) as usize;
        leaf_ppn
            .checked_mul(self.page_size)
            .and_then(|base| base.checked_add(offset))
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

    fn next_heap_for_root(&self) -> VirtualAddress {
        let key = self.satp.get();
        let mut heaps = self.per_root_heap.borrow_mut();
        *heaps.entry(key).or_insert_with(|| VirtualAddress(0))
    }

    fn set_next_heap_for_root(&self, next: VirtualAddress) {
        let key = self.satp.get();
        let mut heaps = self.per_root_heap.borrow_mut();
        heaps.insert(key, next);
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

    /// Allocate space on the per-root heap, map it writable, and copy data.
    pub fn alloc_on_heap(&self, data: &[u8]) -> VirtualAddress {
        let mut addr = self.next_heap_for_root().as_u32();
        let align = 8;
        addr = (addr + (align - 1)) & !(align - 1);
        let end = addr + data.len() as u32;
        let start_va = VirtualAddress(addr);
        self.map_range(start_va, data.len(), Perms::rw_kernel());

        self.copy_into_backing(start_va, data, MemoryAccessKind::Store);
        let end_va = VirtualAddress(end);
        self.set_next_heap_for_root(end_va);
        start_va
    }

    pub fn next_heap(&self) -> VirtualAddress {
        self.next_heap_for_root()
    }

    pub fn set_next_heap(&self, next: VirtualAddress) {
        self.set_next_heap_for_root(next);
    }
}

impl Mmu for Memory {
    fn mem(&self) -> Ref<Vec<u8>> {
        self.backing.borrow()
    }

    fn map_range(&self, start: VirtualAddress, len: usize, perms: Perms) {
        Memory::map_range(self, start, len, perms);
    }

    fn current_root(&self) -> usize {
        self.root_ppn()
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

    fn size(&self) -> usize {
        self.total_size()
    }

    fn offset(&self, addr: VirtualAddress) -> usize {
        addr.as_usize()
    }

    fn set_satp(&self, satp: u32) {
        self.satp.set(satp & SATP_PPN_MASK);
    }

    fn satp(&self) -> u32 {
        self.satp.get()
    }

    fn stack_top(&self) -> VirtualAddress {
        VirtualAddress(self.total_size() as u32)
    }

    fn alloc_on_heap(&self, data: &[u8]) -> VirtualAddress {
        Memory::alloc_on_heap(self, data)
    }

    fn next_heap(&self) -> VirtualAddress {
        self.next_heap_for_root()
    }

    fn set_next_heap(&self, next: VirtualAddress) {
        self.set_next_heap_for_root(next);
    }
}
