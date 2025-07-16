#![no_std]
#![no_main]

extern crate program;

use program::{entrypoint, types::result::Result, vm_panic, require};
use program::types::address::Address;
use program::router::{route};

/// Main entry point for the smart contract.
/// 
/// EDUCATIONAL PURPOSE: This demonstrates a multi-function smart contract
/// that can handle different operations based on a selector. This is a common
/// pattern in smart contract development, similar to how web APIs work.
/// 
/// FUNCTION ROUTING: The contract uses a selector (function ID) to determine
/// which function to call. This allows one contract to provide multiple
/// different operations.
/// 
/// PARAMETERS:
/// - _self_address: The address of this contract (unused in this example)
/// - _caller: The address calling this contract (unused in this example)
/// - data: Binary data containing the function selector and arguments
/// 
/// RETURN VALUE: A Result indicating success/failure and any return data
fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // EDUCATIONAL: Use the router to handle multiple function calls
    // The router decodes the input data and calls the appropriate function
    route(data, _self_address, _caller, |to, from,call| match call.selector {
        0x01 => compare(call.args),    // Function selector 0x01 = compare function
        0x02 => other(call.args),      // Function selector 0x02 = other function
        _ => vm_panic(b"unknown selector"),  // Unknown selector = panic
    })
}

/// Compares two 32-bit integers and returns the larger one.
/// 
/// EDUCATIONAL PURPOSE: This demonstrates how to handle binary data in smart
/// contracts. The function receives raw bytes and must parse them into
/// meaningful data structures.
/// 
/// INPUT FORMAT: 8 bytes total
/// - First 4 bytes: First integer (little-endian)
/// - Last 4 bytes: Second integer (little-endian)
/// 
/// RETURN LOGIC:
/// - If first number > second number: success = true, error_code = first number
/// - If first number <= second number: success = false, error_code = second number
/// 
/// EDUCATIONAL NOTE: The return value uses the Result struct's fields in a
/// non-standard way - error_code actually contains the larger number. This
/// is just for demonstration purposes.
fn compare(data: &[u8]) -> Result {
    // EDUCATIONAL: Validate input size using the require function
    // This demonstrates defensive programming in smart contracts
    require(data.len() == 8, b"compare expects 8 bytes");

    // EDUCATIONAL: Extract the two 32-bit integers from the input data
    let mut a = [0u8; 4];  // Buffer for first integer
    let mut b = [0u8; 4];  // Buffer for second integer
    
    // EDUCATIONAL: Copy bytes from input data to our buffers
    a.copy_from_slice(&data[0..4]);  // First 4 bytes = first integer
    b.copy_from_slice(&data[4..8]);  // Last 4 bytes = second integer

    // EDUCATIONAL: Convert bytes to integers (little-endian format)
    let a = u32::from_le_bytes(a);
    let b = u32::from_le_bytes(b);

    // EDUCATIONAL: Compare the integers and return appropriate result
    if a > b {
        // First number is larger - return success with the larger number
        Result { success: true, error_code: a }
    } else {
        // Second number is larger or equal - return failure with the larger number
        Result { success: false, error_code: b }
    }
}

/// Example function that always fails.
/// 
/// EDUCATIONAL PURPOSE: This demonstrates error handling in smart contracts.
/// Sometimes functions need to fail intentionally (e.g., when conditions
/// aren't met or for testing purposes).
/// 
/// USAGE: This function is called when selector 0x02 is used. It always
/// panics with the message "Intentional failure", which will cause the
/// entire transaction to fail and revert any state changes.
fn other(_data: &[u8]) -> Result {
    // EDUCATIONAL: vm_panic causes the entire transaction to fail
    // This is useful for error conditions that should abort execution
    vm_panic(b"Intentional failure");
}

entrypoint!(my_vm_entry);
