use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::convert::TryInto;

pub struct Memory {
    mem: Rc<RefCell<Vec<u8>>>,
    pub next_heap: Cell<u32>,
}

pub const STACK_OFFSET_FROM_TOP: usize = 0x100;

impl Memory {
    pub fn new(memory_size: usize) -> Self {
        Self {
            mem: Rc::new(RefCell::new(vec![0u8; memory_size])),
            next_heap: Cell::new(0x800), // example starting point
        }
    }

    pub fn mem(&self) -> std::cell::Ref<Vec<u8>> {
        self.mem.borrow()
    }

    pub fn size(&self) -> usize {
        let mem = self.mem();
        mem.len()
    }
    
    pub fn store_u32(&self, addr: usize, val: u32) {
        let mut mem = self.mem.borrow_mut();
        if addr + 4 > mem.len() {
            panic!("store u32 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[addr..addr + 4].copy_from_slice(&val.to_le_bytes());
    }

    pub fn store_u8(&self, addr: usize, val: u8) {
        let mut mem = self.mem.borrow_mut();
        if addr >= mem.len() {
            panic!("store u8 out of bounds: addr = 0x{:08x}", addr);
        }
        mem[addr] = val;
    }

    pub fn load_u32(&self, addr: usize) -> u32 {
        let mem = self.mem.borrow();
        if addr + 4 > mem.len() {
            panic!("load u32 out of bounds: addr = 0x{:08x}", addr);
        }
        u32::from_le_bytes(mem[addr..addr + 4].try_into().unwrap())
    }

    pub fn load_byte(&self, addr: usize) -> u8 {
        let mem = self.mem.borrow();
        mem[addr]
    }

    pub fn load_halfword(&self, addr: usize) -> u16 {
        let mem = self.mem.borrow();
        u16::from_le_bytes(mem[addr..addr + 2].try_into().unwrap())
    }

    pub fn load_word(&self, addr: usize) -> u32 {
        let mem = self.mem.borrow();
        u32::from_le_bytes(mem[addr..addr + 4].try_into().unwrap())
    }

    pub fn store_byte(&mut self, addr: usize, value: u8) {
        let mut mem = self.mem.borrow_mut();
        mem[addr] = value;
    }

    pub fn store_halfword(&mut self, addr: usize, value: u16) {
        let mut mem = self.mem.borrow_mut();
        mem[addr..addr + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn store_word(&mut self, addr: usize, value: u32) {
        let mut mem = self.mem.borrow_mut();
        mem[addr..addr + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn mem_slice(&self, start: usize, end: usize) -> Option<std::cell::Ref<[u8]>> {
        let mem_ref = self.mem.borrow();
        // Return a `Ref<[u8]>` if you want to use it directly
        // or clone if needed outside the borrow context
        if end > mem_ref.len() || start > end {
            return None;
        }
        Some(std::cell::Ref::map(mem_ref, move |v| &v[start..end]))
    }

    pub fn write_code(&self, code: &[u8]) {
        self.mem.borrow_mut()[..code.len()].copy_from_slice(code);
    }

    pub fn alloc_on_heap(&self, data: &[u8]) -> u32 {
        let addr = self.next_heap.get();
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
