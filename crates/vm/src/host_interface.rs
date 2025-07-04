use std::fmt::Debug;

pub trait HostInterface: Debug {
    fn call_contract(&mut self, from: [u8; 20], to: [u8; 20], input_data: Vec<u8>) -> u32;
} 