#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, require};
use program::types::address::Address;
use program::call::call;

// Include the auto-generated ABI client code for calculator
include!("../bin/calculator_abi.rs");

/// Client program that calls the calculator contract using auto-generated ABI
fn my_vm_entry(_self_address: Address, caller: Address, data: &[u8]) -> Result {
    // Parse input: calculator address (20 bytes) + operation (1 byte) + two u32 values (8 bytes)
    require(data.len() >= 29, b"Need address, operation, and two values");
    
    // Extract calculator contract address
    let calculator_addr = Address::from_ptr(&data[..20]).expect("Invalid address");
    
    // Create calculator client using auto-generated code
    let calculator = CalculatorContract::new(calculator_addr);
    
    // Extract operation selector
    let operation = data[20];
    
    // Extract operands
    let mut a_bytes = [0u8; 4];
    let mut b_bytes = [0u8; 4];
    a_bytes.copy_from_slice(&data[21..25]);
    b_bytes.copy_from_slice(&data[25..29]);
    
    let a = u32::from_le_bytes(a_bytes);
    let b = u32::from_le_bytes(b_bytes);
    
    // Encode the two u32 values as bytes for the calculator functions
    let mut calc_data = [0u8; 8];
    calc_data[0..4].copy_from_slice(&a.to_le_bytes());
    calc_data[4..8].copy_from_slice(&b.to_le_bytes());
    
    // Call the appropriate calculator function using the generated client
    let result = match operation {
        1 => calculator.add(&caller, &calc_data),
        2 => calculator.subtract(&caller, &calc_data),
        3 => calculator.multiply(&caller, &calc_data),
        4 => calculator.divide(&caller, &calc_data),
        _ => return Result::new(false, 2), // Invalid operation
    };
    
    // Return the result from the calculator
    match result {
        Some(r) => r,
        None => Result::new(false, 3), // Calculator call failed
    }
}

entrypoint!(my_vm_entry);