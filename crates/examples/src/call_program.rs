#![no_std]
#![no_main]

extern crate program;

use program::{entrypoint, types::result::Result, require, vm_panic, DataParser};
use program::call::call;
use program::types::address::Address;

// Include the auto-generated ABI client code for simple program
include!("../bin/simple_abi.rs");

/// Program that uses generated ABI code to call the simple contract
/// 
/// This demonstrates using auto-generated client code instead of manual encoding.
/// The program expects:
/// - 20 bytes: Address of the simple contract
/// - 8 bytes: Two u32 values to compare (4 bytes each)
fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Ensure there's enough data
    require(data.len() == 28, b"input data must be 28 bytes");

    // Parse the simple contract address from the first 20 bytes
    let mut parser = DataParser::new(data);
    let simple_addr = parser.read_address();

    // Create the client using generated code
    let simple_client = SimpleContract::new(simple_addr);

    // Extract the two numbers to compare
    let first = parser.read_u32();
    let second = parser.read_u32();
    
    // Prepare the data for the simple contract (8 bytes: two u32 values)
    let mut call_data = [0u8; 8];
    call_data[0..4].copy_from_slice(&first.to_le_bytes());
    call_data[4..8].copy_from_slice(&second.to_le_bytes());
    
    // Call the simple contract using the generated client's call_main method
    let ret = match simple_client.call_main(&_caller, &call_data) {
        Some(result) => result,
        None => vm_panic(b"program call failed"),
    };
    
    ret
}

// Register the function as the contract's entrypoint
entrypoint!(my_vm_entry);
