#![no_std]
#![no_main]

extern crate contract_std; // Required for macro visibility
use contract_std::contract;

contract! {
    pub fn add_to_base(base: u32, value: u32) -> u32 {
        base + value
    }

    pub fn is_positive(val: u32) -> u32 {
        if val > 0 { 1 } else { 0 }
    }
}
