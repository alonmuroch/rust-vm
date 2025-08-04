use std::fmt::Debug;

pub trait HostInterface: Debug {
    // calls another program, returns result ptr and page index
    fn call_program(&mut self, from: [u8; 20], to: [u8; 20], input_data: Vec<u8>) -> (u32, usize);
    fn read_memory_page(&mut self, page_index: usize, guest_ptr: u32, len: usize) -> Option<Vec<u8>>;
    fn fire_event(&mut self, event: Vec<u8>);
}

#[derive(Debug)]
pub struct NoopHost;

impl HostInterface for NoopHost {
    fn call_program(&mut self, _from: [u8; 20], _to: [u8; 20], _input_data: Vec<u8>) -> (u32, usize) {
        (0, 0)
    }
    fn read_memory_page(&mut self, _page_index: usize, _guest_ptr: u32, _len: usize) -> Option<Vec<u8>> {
        None
    }
    fn fire_event(&mut self, _event: Vec<u8>) {
        // No operation
    }
} 