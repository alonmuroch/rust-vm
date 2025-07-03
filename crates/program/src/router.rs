/// Represents a function call with a selector (function ID) and arguments.
/// This is the core data structure for routing function calls in our VM.
/// 
/// EDUCATIONAL NOTE: In virtual machines, we often need to call different functions
/// based on a selector (like a function ID). This struct holds the selector and
/// the raw bytes that represent the function arguments. The lifetime 'a ensures
/// the args reference stays valid as long as the input data exists.
#[derive(Debug, Clone, Copy)]
pub struct FuncCall<'a> {
    /// The function selector (0-255) that identifies which function to call
    pub selector: u8,
    /// Raw bytes containing the function arguments
    pub args: &'a [u8],
}

use crate::{Result, vm_panic};

/// Decodes a sequence of function calls from a binary input buffer.
/// 
/// EDUCATIONAL PURPOSE: This function demonstrates how to parse a binary protocol
/// where multiple function calls are packed into a single buffer. This is common
/// in blockchain VMs and RPC systems.
/// 
/// BINARY FORMAT EXPLANATION:
/// Each function call is encoded as: [selector: u8][arg_len: u8][args: arg_len bytes]
/// - selector: Which function to call (0-255)
/// - arg_len: How many bytes of arguments follow
/// - args: The actual argument data
/// 
/// SAFETY CONSIDERATIONS:
/// - We validate buffer bounds to prevent out-of-bounds access
/// - We check for incomplete headers and malformed data
/// - We limit the number of calls to prevent buffer overflow
/// 
/// PARAMETERS:
/// - input: The binary buffer containing encoded function calls
/// - buffer: Pre-allocated array to store decoded FuncCall structs
/// 
/// RETURNS: Number of successfully decoded function calls
pub fn decode_calls<'a>(mut input: &'a [u8], buffer: &mut [FuncCall<'a>]) -> usize {
    let mut count = 0;

    // Process each function call in the input buffer
    while !input.is_empty() {
        // EDUCATIONAL: Always check buffer bounds before accessing data
        // This prevents segmentation faults and security vulnerabilities
        if input.len() < 2 {
            vm_panic(b"decode: incomplete header");
        }

        // Extract the function selector and argument length
        let selector = input[0];
        let arg_len = input[1] as usize;

        // EDUCATIONAL: Validate that we have enough data for the arguments
        // This is crucial for preventing buffer overruns
        if input.len() < 2 + arg_len {
            vm_panic(b"decode: args too short");
        }

        // EDUCATIONAL: Prevent buffer overflow by checking our output buffer capacity
        if count >= buffer.len() {
            vm_panic(b"decode: too many calls");
        }

        // Extract the argument bytes (slice from position 2 to 2+arg_len)
        let args = &input[2..2 + arg_len];

        // Store the decoded function call in our buffer
        buffer[count] = FuncCall {
            selector,
            args,
        };

        count += 1;
        // Move to the next function call in the input buffer
        input = &input[2 + arg_len..];
    }

    count
}

/// Generic router that takes the input buffer and a closure to dispatch each call.
/// 
/// EDUCATIONAL PURPOSE: This function demonstrates the power of Rust's generic
/// programming and closures. It's a higher-order function that can work with
/// any handler function you provide.
/// 
/// DESIGN PATTERN: This implements the "Strategy Pattern" where the routing
/// logic is separated from the actual function handling logic. This makes the
/// code more modular and testable.
/// 
/// MEMORY MANAGEMENT: We use a fixed-size array to avoid heap allocations,
/// which is important in embedded systems and VMs where memory is constrained.
/// 
/// ERROR HANDLING: Each function call can succeed or fail, but we continue
/// processing all calls and return the result of the last one. This ensures
/// that one bad function call doesn't stop the entire batch.
/// 
/// PARAMETERS:
/// - input: Binary buffer containing encoded function calls
/// - max_calls: Maximum number of calls to process (safety limit)
/// - handler: Closure that processes each individual function call
/// 
/// RETURNS: Result of the last processed function call
pub fn route<'a>(
    input: &'a [u8],
    max_calls: usize,
    mut handler: impl FnMut(&FuncCall<'a>) -> Result,
) -> Result {
    // EDUCATIONAL: Fixed-size array for storing decoded calls
    // This avoids heap allocations and provides predictable memory usage
    let mut buf: [Option<FuncCall<'a>>; 8] = [None, None, None, None, None, None, None, None];
    let mut input = input;
    let mut count = 0;

    // First pass: decode all function calls into our buffer
    while !input.is_empty() && count < max_calls {
        // EDUCATIONAL: Same safety checks as in decode_calls
        if input.len() < 2 {
            vm_panic(b"router: bad header");
        }

        let selector = input[0];
        let arg_len = input[1] as usize;

        if input.len() < 2 + arg_len {
            vm_panic(b"router: bad arg len");
        }

        let args = &input[2..2 + arg_len];

        // Store the decoded call in our buffer
        buf[count] = Some(FuncCall { selector, args });
        count += 1;
        input = &input[2 + arg_len..];
    }

    // EDUCATIONAL: Initialize with a default success result
    // This ensures we always return a valid Result even if no calls are processed
    let mut last_result = Result {
        success: true,
        error_code: 0,
    };

    // Second pass: execute each function call using the provided handler
    for i in 0..count {
        if let Some(call) = &buf[i] {
            // EDUCATIONAL: Each call can succeed or fail, but we continue processing
            // This is important for batch operations where you want to process all calls
            last_result = handler(call);
        }
    }

    last_result
}