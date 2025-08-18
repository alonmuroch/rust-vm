#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, logf, log, concat_str};
use program::types::address::Address;

/// Comprehensive logging demonstration showing all format specifiers
unsafe fn logging(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Simple string logging
    logf!("=== Logging Demo Started ===");
    
    // Integer formats
    let num = 42;
    logf!("Decimal: %d", num);
    logf!("Unsigned: %u", num);
    logf!("Hexadecimal: %x", 0xDEADBEEF);
    
    // Multiple values in one log
    let x = 10;
    let y = 20;
    logf!("x=%d, y=%d, sum=%d", x, y, x + y);
    
    // Character logging
    let ch = 'A' as u32;
    logf!("Character: %c", ch);
    
    // Floating point
    let pi_bits = 3.14159f32.to_bits();
    logf!("Pi approximation: %f", pi_bits);
    
    // String logging - now simplified!
    let msg = b"Hello, VM!";
    log!("Message: %s", msg);
    
    // String concatenation - requires a buffer in no_std (even though we have an allocator)
    let mut buffer = [0u8; 64];  // Stack-allocated storage
    let greeting = concat_str!(buffer, b"Hello, ", b"World", b"!");
    log!("Concatenated: %s", greeting);
    
    
    // Byte array logging (hex format) - simplified!
    let bytes = [0xDE, 0xAD, 0xBE, 0xEF];
    log!("Bytes (hex): %b", bytes);
    
    // Array of u32s - simplified!
    let numbers = [1u32, 2, 3, 4, 5];
    log!("Numbers: %a", numbers);
    
    // Array of u8s (decimal format) - simplified!
    let bytes_decimal = [10u8, 20, 30, 40, 50];
    log!("Bytes (decimal): %A", bytes_decimal);
    
    // Process input data
    if data.len() >= 4 {
        let mut val_bytes = [0u8; 4];
        val_bytes.copy_from_slice(&data[0..4]);
        let value = u32::from_le_bytes(val_bytes);
        logf!("Input value: %d (0x%x)", value, value);
        
        // Log remaining bytes if any
        if data.len() > 4 {
            let remaining = &data[4..];
            logf!("Remaining %d bytes: ", remaining.len() as u32);
            log!("%b", remaining);
        }
    } else {
        logf!("Input too short: %d bytes", data.len() as u32);
    }
    
    // Demonstrate escape sequence
    logf!("100%% complete!");
    
    // Complex example with mixed types
    let score = 95;
    let grade = 'A' as u32;
    let bonus = 5;
    logf!("Score: %d + Bonus: %d = Total: %d", 
          score, bonus, score + bonus);
    logf!("Grade: %c", grade);
    
    // Large array (partial display for efficiency) - simplified!
    let large_array = [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    log!("Large array: %a", large_array);
    
    logf!("=== Logging Demo Complete ===");
    
    Result::new(true, 0)
}

entrypoint!(logging);