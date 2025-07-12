use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct MemoryPage {
    mem: Rc<RefCell<Vec<u8>>>,
    pub next_heap: Cell<u32>,
    pub base_address: usize, // New: base address for guest memory mapping
}

pub const HEAP_PTR_OFFSET: u32 = 0x100;

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

    pub fn mem(&self) -> std::cell::Ref<Vec<u8>> {
        self.mem.borrow()
    }

    pub fn size(&self) -> usize {
        let mem = self.mem();
        mem.len()
    }

    pub fn offset(&self, addr: usize) -> usize {
        addr.checked_sub(self.base_address).expect("Address below base_address")
    }

    pub fn store_u16(&self, addr: usize, val: u16) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset + 2 > mem.len() {
            panic!("store u16 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
    }
    
    pub fn store_u32(&self, addr: usize, val: u32) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset + 4 > mem.len() {
            panic!("store u32 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    pub fn store_u8(&self, addr: usize, val: u8) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        if offset >= mem.len() {
            panic!("store u8 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[offset] = val;
    }

    pub fn load_u32(&self, addr: usize) -> u32 {
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        if offset + 4 > mem.len() {
            panic!("load u32 out of bounds: addr = 0x{:08x}", addr);
        }
        u32::from_le_bytes(mem[offset..offset + 4].try_into().unwrap())
    }

    pub fn load_byte(&self, addr: usize) -> u8 {
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        mem[offset]
    }

    pub fn load_halfword(&self, addr: usize) -> u16 {
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        u16::from_le_bytes(mem[offset..offset + 2].try_into().unwrap())
    }

    pub fn load_word(&self, addr: usize) -> u32 {
        let offset = self.offset(addr);
        let mem = self.mem.borrow();
        u32::from_le_bytes(mem[offset..offset + 4].try_into().unwrap())
    }

    pub fn store_byte(&mut self, addr: usize, value: u8) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        mem[offset] = value;
    }

    pub fn store_halfword(&mut self, addr: usize, value: u16) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        mem[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn store_word(&mut self, addr: usize, value: u32) {
        let offset = self.offset(addr);
        let mut mem = self.mem.borrow_mut();
        mem[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn mem_slice(&self, start: usize, end: usize) -> Option<std::cell::Ref<[u8]>> {
        let start_offset = self.offset(start);
        let end_offset = self.offset(end);
        let mem_ref = self.mem.borrow();
        if end_offset > mem_ref.len() || start_offset > end_offset {
            return None;
        }
        Some(std::cell::Ref::map(mem_ref, move |v| &v[start_offset..end_offset]))
    }

    pub fn write_code(&mut self, start_addr: usize, code: &[u8]) {
        let start_offset = self.offset(start_addr);
        let mut mem = self.mem.borrow_mut();
        let end = start_offset + code.len();
        mem[start_offset..end].copy_from_slice(code);

        // set heap pointer
        self.next_heap = Cell::new(start_offset as u32 + code.len() as u32 + HEAP_PTR_OFFSET);
    }

    pub fn alloc_on_heap(&self, data: &[u8]) -> u32 {
        let mut addr = self.next_heap.get();

        // Align to 4 bytes (or 8 if you're storing u64s)
        let align = 8;
        addr = (addr + (align - 1)) & !(align - 1);
        
        let end = addr + data.len() as u32;
        assert!(end as usize <= self.size());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_zero_base() {
        let mem = MemoryPage::new_with_base(1024, 0);
        assert_eq!(mem.offset(0), 0);
        assert_eq!(mem.offset(100), 100);
        assert_eq!(mem.offset(1023), 1023);
    }

    #[test]
    fn test_offset_high_base() {
        let base = 0x80000000;
        let mem = MemoryPage::new_with_base(1024, base);
        assert_eq!(mem.offset(base), 0);
        assert_eq!(mem.offset(base + 100), 100);
        assert_eq!(mem.offset(base + 1023), 1023);
    }

    #[test]
    #[should_panic(expected = "Address below base_address")]
    fn test_offset_below_base_panics() {
        let base = 0x80000000;
        let mem = MemoryPage::new_with_base(1024, base);
        mem.offset(base - 1);
    }
}
