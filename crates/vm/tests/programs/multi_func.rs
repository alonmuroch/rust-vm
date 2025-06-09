#![no_std]
#![no_main]

use core::convert::TryInto;

extern crate program; // Required for macro visibility
use program::{entrypoint, Pubkey, Result};

fn my_vm_entry(caller: Pubkey, data: &[u8]) -> Result {
    if data.len() < 16 {
        return Result { success: false, error_code: 1 }; // Not enough data
    }

    let first = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let second = u64::from_le_bytes(data[8..16].try_into().unwrap());

    if first > second {
        Result { success: true, error_code: 0 }
    } else {
        Result { success: false, error_code: 0 }
    }
}

// Register the function as the entrypoint
entrypoint!(my_vm_entry);