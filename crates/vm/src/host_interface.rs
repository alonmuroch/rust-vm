use std::fmt::Debug;

pub trait HostInterface: Debug {
    // calls another program, returns result ptr and page index
    fn call_program(&mut self, from: [u8; 20], to: [u8; 20], input_data: Vec<u8>) -> (u32, usize);
    fn read_memory_page(&mut self, page_index: usize, guest_ptr: u32, len: usize) -> Option<Vec<u8>>;
} 