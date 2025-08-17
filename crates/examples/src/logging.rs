#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, logf, log_array};
use program::types::address::Address;

/// Comprehensive logging demonstration showing all format specifiers
fn logging(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Simple string logging
    logf!(b"=== Logging Demo Started ===");
    
    // Integer formats
    let num = 42;
    logf!(b"Decimal: %d", num);
    logf!(b"Unsigned: %u", num);
    logf!(b"Hexadecimal: %x", 0xDEADBEEF);
    
    // Multiple values in one log
    let x = 10;
    let y = 20;
    logf!(b"x=%d, y=%d, sum=%d", x, y, x + y);
    
    // Character logging
    let ch = 'A' as u32;
    logf!(b"Character: %c", ch);
    
    // Floating point
    let pi_bits = 3.14159f32.to_bits();
    logf!(b"Pi approximation: %f", pi_bits);
    
    // String logging (requires pointer and length)
    let msg = b"Hello, VM!";
    logf!(b"Message: %s", msg.as_ptr() as u32, msg.len() as u32);
    
    // Byte array logging (hex format)
    let bytes = [0xDE, 0xAD, 0xBE, 0xEF];
    logf!(b"Bytes (hex): %b", bytes.as_ptr() as u32, bytes.len() as u32);
    
    // Array of u32s
    let numbers = [1u32, 2, 3, 4, 5];
    log_array!(b"Numbers: %a", &numbers);
    
    // Array of u8s (decimal format)
    let bytes_decimal = [10u8, 20, 30, 40, 50];
    logf!(b"Bytes (decimal): %A", bytes_decimal.as_ptr() as u32, bytes_decimal.len() as u32);
    
    // Process input data
    if data.len() >= 4 {
        let mut val_bytes = [0u8; 4];
        val_bytes.copy_from_slice(&data[0..4]);
        let value = u32::from_le_bytes(val_bytes);
        logf!(b"Input value: %d (0x%x)", value, value);
        
        // Log remaining bytes if any
        if data.len() > 4 {
            let remaining = &data[4..];
            logf!(b"Remaining %d bytes: %b", 
                  remaining.len() as u32,
                  remaining.as_ptr() as u32, 
                  remaining.len() as u32);
        }
    } else {
        logf!(b"Input too short: %d bytes", data.len() as u32);
    }
    
    // Demonstrate escape sequence
    logf!(b"100%% complete!");
    
    // Complex example with mixed types
    let score = 95;
    let grade = 'A' as u32;
    let bonus = 5;
    logf!(b"Score: %d + Bonus: %d = Total: %d", 
          score, bonus, score + bonus);
    logf!(b"Grade: %c", grade);
    
    // Large array (partial display for efficiency)
    let large_array = [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    log_array!(b"Large array: %a", &large_array);
    
    logf!(b"=== Logging Demo Complete ===");
    
    Result::new(true, 0)
}

entrypoint!(logging);