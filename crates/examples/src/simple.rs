#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Pubkey, Result};

fn my_vm_entry(_caller: Pubkey, data: &[u8]) -> Result {
    if data.len() < 16 {
        return Result { success: false, error_code: 1 }; // Not enough data
    }

    let mut first_bytes = [0u8; 8];
    let mut second_bytes = [0u8; 8];

    first_bytes.copy_from_slice(&data[0..8]);
    second_bytes.copy_from_slice(&data[8..16]);

    let first = u64::from_le_bytes(first_bytes);
    let second = u64::from_le_bytes(second_bytes);

    let error_code = if first > second { 1 } else { 2 };
    Result { success: true, error_code }
}

// Register the function as the entrypoint
entrypoint!(my_vm_entry);
