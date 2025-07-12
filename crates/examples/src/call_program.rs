#![no_std]
#![no_main]

extern crate program;
use core::ops::Add;

use program::{entrypoint, types::result::Result, require, logf, types::O};
use program::call::call;
use program::types::address::Address;

fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Ensure there's enough data to extract an address
    require(data.len() == 28, b"input data must be 28 bytes"); // Error code 1: invalid input length

    // Parse the address from the first 20 bytes
    let target = Address::from_ptr(&data[..20]).expect("Invalid address format");

    // Pass the remaining bytes as input
    let input = &data[20..];
    let ret = match call(&_caller, &target, input) {
        Some(result) => result,
        None => Result { success: false, error_code: 0}, // Or another error-handling path
    };
    ret
}

// Register the function as the contract's entrypoint
entrypoint!(my_vm_entry);
