#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Result, logf, require};
use program::types::address::Address;    

fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    require(data.len() == 8, b"Input data must be at least 8 bytes long");

    let mut first_bytes = [0u8; 4];
    let mut second_bytes = [0u8; 4];
    first_bytes.copy_from_slice(&data[0..4]);
    second_bytes.copy_from_slice(&data[4..8]);

    let first = u32::from_le_bytes(first_bytes);
    let second = u32::from_le_bytes(second_bytes);

    let error_code = if first > second { 
        return Result { success: true, error_code:first as u32}
    } else {
        return Result { success: false, error_code: second as u32}
    };
}

// Register the function as the entrypoint
entrypoint!(my_vm_entry);
