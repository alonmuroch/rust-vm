#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, require};
use program::types::address::Address;    

/// Simple smart contract that compares two 32-bit integers.
/// 
/// EDUCATIONAL PURPOSE: This demonstrates a basic smart contract that:
/// - Accepts input data (two 32-bit integers)
/// - Performs a simple comparison operation
/// - Returns a result indicating success/failure
/// 
/// CONTRACT BEHAVIOR:
/// - Takes 8 bytes of input data (two 32-bit integers)
/// - Compares the first integer with the second
/// - Returns success=true if first > second, success=false otherwise
/// - Uses the larger value as the error_code
/// 
/// INPUT FORMAT: The contract expects exactly 8 bytes:
/// - Bytes 0-3: First 32-bit integer (little-endian)
/// - Bytes 4-7: Second 32-bit integer (little-endian)
/// 
/// OUTPUT FORMAT: Returns a Result struct with:
/// - success: true if first > second, false otherwise
/// - error_code: The larger of the two input values
/// 
/// REAL-WORLD USAGE: This type of contract could be used for:
/// - Simple validation logic
/// - Conditional execution based on input values
/// - Basic decision-making in decentralized applications
/// 
/// SECURITY CONSIDERATIONS:
/// - Input validation prevents buffer overflows
/// - No external calls or state modifications
/// - Deterministic execution for all inputs
fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // EDUCATIONAL: Validate input data length to prevent buffer overflows
    // This is a critical security practice in smart contracts
    require(data.len() == 8, b"Input data must be at least 8 bytes long");

    // EDUCATIONAL: Extract two 32-bit integers from the input data
    // This demonstrates how to parse binary data in smart contracts
    let mut first_bytes = [0u8; 4];
    let mut second_bytes = [0u8; 4];
    first_bytes.copy_from_slice(&data[0..4]);  // First 4 bytes
    second_bytes.copy_from_slice(&data[4..8]); // Second 4 bytes

    // EDUCATIONAL: Convert bytes to integers using little-endian format
    // This matches the VM's memory layout and is common in embedded systems
    let first = u32::from_le_bytes(first_bytes);
    let second = u32::from_le_bytes(second_bytes);

    // EDUCATIONAL: Perform the comparison and return appropriate result
    // This demonstrates conditional logic in smart contracts
    if first > second { 
        return Result { success: true, error_code:first as u32}
    } else {
        return Result { success: false, error_code: second as u32}
    };
}

// EDUCATIONAL: Register the function as the entrypoint
// This macro tells the VM which function to call when the contract is invoked
entrypoint!(my_vm_entry);
