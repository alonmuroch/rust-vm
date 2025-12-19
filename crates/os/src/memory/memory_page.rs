use std::cell::{Cell, Ref, RefCell};
use std::convert::TryInto;
use std::rc::Rc;
use vm::memory::{HEAP_PTR_OFFSET, memory};
use vm::metering::{MemoryAccessKind, MeterResult, Metering};

#[derive(Debug, Clone)]
pub struct MemoryPage {
    mem: Rc<RefCell<Vec<u8>>>,
    pub next_heap: Cell<u32>,
    pub base_address: usize, // base address for guest memory mapping
}

impl MemoryPage {
    pub fn new_with_base(memory_size: usize, base_address: usize) -> Self {
        Self {
            mem: Rc::new(RefCell::new(vec![0u8; memory_size])),
            next_heap: Cell::new(0),
            base_address,
        }
    }

    pub fn new(memory_size: usize) -> Self {
        Self::new_with_base(memory_size, 0)
    }

    pub fn mem(&self) -> Ref<Vec<u8>> {
        self.mem.borrow()
    }

    pub fn size(&self) -> usize {
        let mem = self.mem();
        mem.len()
    }

    pub fn offset(&self, addr: usize) -> usize {
        addr.checked_sub(self.base_address)
            .expect("Address below base_address")
    }

    fn meter_access(
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
        addr: usize,
        bytes: usize,
    ) -> bool {
        matches!(
            metering.on_memory_access(kind, addr, bytes),
            MeterResult::Continue
        )
    }

    pub fn store_u16(
        &self,
        addr: usize,
        val: u16,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 2) {
            return false;
        }
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset + 2 > mem.len() {
            panic!("store u16 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
        true
    }

    pub fn store_u32(
        &self,
        addr: usize,
        val: u32,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 4) {
            return false;
        }
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset + 4 > mem.len() {
            panic!("store u32 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        true
    }

    pub fn store_u8(
        &self,
        addr: usize,
        val: u8,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        if !Self::meter_access(metering, kind, addr, 1) {
            return false;
        }
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset >= mem.len() {
            panic!("store u8 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset] = val;
        true
    }

    pub fn load_u32(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        if !Self::meter_access(metering, kind, addr, 4) {
            return None;
        }
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        if offset + 4 > mem.len() {
            panic!("load u32 out of bounds: addr = 0x{:08x}", addr);
        }
        Some(u32::from_le_bytes(
            mem[offset..offset + 4].try_into().unwrap(),
        ))
    }

    pub fn load_byte(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u8> {
        if !Self::meter_access(metering, kind, addr, 1) {
            return None;
        }
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        Some(mem[offset])
    }

    pub fn load_halfword(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u16> {
        if !Self::meter_access(metering, kind, addr, 2) {
            return None;
        }
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        Some(u16::from_le_bytes(
            mem[offset..offset + 2].try_into().unwrap(),
        ))
    }

    pub fn load_word(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        if !Self::meter_access(metering, kind, addr, 4) {
            return None;
        }
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        Some(u32::from_le_bytes(
            mem[offset..offset + 4].try_into().unwrap(),
        ))
    }

    pub fn mem_slice(&self, start: usize, end: usize) -> Option<std::cell::Ref<[u8]>> {
        let start_offset = self.offset(start);
        let end_offset = self.offset(end);
        let mem_ref = self.mem.borrow();
        if end_offset > mem_ref.len() || start_offset > end_offset {
            return None;
        }
        Some(std::cell::Ref::map(mem_ref, move |v| {
            &v[start_offset..end_offset]
        }))
    }

    pub fn write_code(&self, start_addr: usize, code: &[u8]) {
        let start_offset = self.offset(start_addr);
        let mut mem = self.mem.borrow_mut();
        let end = start_offset + code.len();
        mem[start_offset..end].copy_from_slice(code);

        // set heap pointer
        self.next_heap
            .set(start_offset as u32 + code.len() as u32 + HEAP_PTR_OFFSET);
    }

    pub fn alloc_on_heap(&self, data: &[u8]) -> u32 {
        let mut addr = self.next_heap.get();

        // Align to 4 bytes (or 8 if you're storing u64s)
        let align = 8;
        addr = (addr + (align - 1)) & !(align - 1);

        let end = addr + data.len() as u32;
        assert!(
            end as usize <= self.size(),
            "Out of memory: trying to allocate {} bytes, but only {} bytes available",
            data.len(),
            self.size() - addr as usize
        );

        self.mem.borrow_mut()[addr as usize..end as usize].copy_from_slice(data);
        self.next_heap.set(end);

        addr
    }

    pub fn stack_top(&self) -> u32 {
        self.size() as u32
    }
}

impl Default for MemoryPage {
    fn default() -> Self {
        MemoryPage::new(4096)
    }
}

impl memory for MemoryPage {
    fn mem(&self) -> Ref<Vec<u8>> {
        MemoryPage::mem(self)
    }

    fn mem_slice(&self, start: usize, end: usize) -> Option<std::cell::Ref<[u8]>> {
        MemoryPage::mem_slice(self, start, end)
    }

    fn store_u16(
        &self,
        addr: usize,
        val: u16,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        MemoryPage::store_u16(self, addr, val, metering, kind)
    }

    fn store_u32(
        &self,
        addr: usize,
        val: u32,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        MemoryPage::store_u32(self, addr, val, metering, kind)
    }

    fn store_u8(
        &self,
        addr: usize,
        val: u8,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> bool {
        MemoryPage::store_u8(self, addr, val, metering, kind)
    }

    fn load_u32(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        MemoryPage::load_u32(self, addr, metering, kind)
    }

    fn load_byte(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u8> {
        MemoryPage::load_byte(self, addr, metering, kind)
    }

    fn load_halfword(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u16> {
        MemoryPage::load_halfword(self, addr, metering, kind)
    }

    fn load_word(
        &self,
        addr: usize,
        metering: &mut dyn Metering,
        kind: MemoryAccessKind,
    ) -> Option<u32> {
        MemoryPage::load_word(self, addr, metering, kind)
    }

    fn write_code(&self, start_addr: usize, code: &[u8]) {
        MemoryPage::write_code(self, start_addr, code)
    }

    fn alloc_on_heap(&self, data: &[u8]) -> u32 {
        MemoryPage::alloc_on_heap(self, data)
    }

    fn stack_top(&self) -> u32 {
        MemoryPage::stack_top(self)
    }

    fn size(&self) -> usize {
        MemoryPage::size(self)
    }

    fn offset(&self, addr: usize) -> usize {
        MemoryPage::offset(self, addr)
    }

    fn next_heap(&self) -> u32 {
        self.next_heap.get()
    }

    fn set_next_heap(&self, next: u32) {
        self.next_heap.set(next);
    }
}
