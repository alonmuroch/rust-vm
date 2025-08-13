#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, require};
use program::router::route;
use program::types::address::Address;

/// Calculator contract with multiple math operations
/// Each function is called via a selector in the router

fn add(data: &[u8]) -> Result {
    require(data.len() >= 8, b"Need two u32 values");
    
    let mut a_bytes = [0u8; 4];
    let mut b_bytes = [0u8; 4];
    a_bytes.copy_from_slice(&data[0..4]);
    b_bytes.copy_from_slice(&data[4..8]);
    
    let a = u32::from_le_bytes(a_bytes);
    let b = u32::from_le_bytes(b_bytes);
    let result = a + b;
    
    Result::with_u32(result)
}

fn subtract(data: &[u8]) -> Result {
    require(data.len() >= 8, b"Need two u32 values");
    
    let mut a_bytes = [0u8; 4];
    let mut b_bytes = [0u8; 4];
    a_bytes.copy_from_slice(&data[0..4]);
    b_bytes.copy_from_slice(&data[4..8]);
    
    let a = u32::from_le_bytes(a_bytes);
    let b = u32::from_le_bytes(b_bytes);
    let result = a.saturating_sub(b);
    
    Result::with_u32(result)
}

fn multiply(data: &[u8]) -> Result {
    require(data.len() >= 8, b"Need two u32 values");
    
    let mut a_bytes = [0u8; 4];
    let mut b_bytes = [0u8; 4];
    a_bytes.copy_from_slice(&data[0..4]);
    b_bytes.copy_from_slice(&data[4..8]);
    
    let a = u32::from_le_bytes(a_bytes);
    let b = u32::from_le_bytes(b_bytes);
    let result = a.saturating_mul(b);
    
    Result::with_u32(result)
}

fn divide(data: &[u8]) -> Result {
    require(data.len() >= 8, b"Need two u32 values");
    
    let mut a_bytes = [0u8; 4];
    let mut b_bytes = [0u8; 4];
    a_bytes.copy_from_slice(&data[0..4]);
    b_bytes.copy_from_slice(&data[4..8]);
    
    let a = u32::from_le_bytes(a_bytes);
    let b = u32::from_le_bytes(b_bytes);
    
    require(b != 0, b"Division by zero");
    let result = a / b;
    
    Result::with_u32(result)
}

fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    route(data, _self_address, _caller, |_to, _from, call| match call.selector {
        0x01 => add(call.args),
        0x02 => subtract(call.args),
        0x03 => multiply(call.args),
        0x04 => divide(call.args),
        _ => Result::new(false, 1), // Unknown function selector
    })
}

entrypoint!(my_vm_entry);