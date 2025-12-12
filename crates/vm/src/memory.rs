use std::cell::Ref;
use std::rc::Rc;
use crate::metering::{Metering, MemoryAccessKind};

pub const HEAP_PTR_OFFSET: u32 = 0x100;

pub trait Memory: std::fmt::Debug {
    fn mem(&self) -> Ref<Vec<u8>>;
    fn mem_slice(&self, start: usize, end: usize) -> Option<std::cell::Ref<[u8]>>;
    fn store_u16(&self, addr: usize, val: u16, metering: &mut dyn Metering, kind: MemoryAccessKind) -> bool;
    fn store_u32(&self, addr: usize, val: u32, metering: &mut dyn Metering, kind: MemoryAccessKind) -> bool;
    fn store_u8(&self, addr: usize, val: u8, metering: &mut dyn Metering, kind: MemoryAccessKind) -> bool;
    fn load_u32(&self, addr: usize, metering: &mut dyn Metering, kind: MemoryAccessKind) -> Option<u32>;
    fn load_byte(&self, addr: usize, metering: &mut dyn Metering, kind: MemoryAccessKind) -> Option<u8>;
    fn load_halfword(&self, addr: usize, metering: &mut dyn Metering, kind: MemoryAccessKind) -> Option<u16>;
    fn load_word(&self, addr: usize, metering: &mut dyn Metering, kind: MemoryAccessKind) -> Option<u32>;
    fn write_code(&self, start_addr: usize, code: &[u8]);
    fn alloc_on_heap(&self, data: &[u8]) -> u32;
    fn stack_top(&self) -> u32;
    fn size(&self) -> usize;
    fn offset(&self, addr: usize) -> usize;
    fn next_heap(&self) -> u32;
    fn set_next_heap(&self, next: u32);
}

pub type SharedMemory = Rc<dyn Memory>;
