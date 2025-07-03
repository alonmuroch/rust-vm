#![no_std]

#[cfg(test)]
extern crate std;

/* --------------------------------- Imports --------------------------------- */

// External crates
pub extern crate hex;
pub extern crate types;

/* --------------------------------- Modules --------------------------------- */

// Result module
pub mod result;
pub use result::Result;

// Logging macros
pub mod log;

// Entrypoint macro
#[macro_use]
pub mod entrypoint;

// Persistent storage system
#[macro_use]
pub mod storage;
pub use storage::Persistent; // Allow `$crate::Persistent` in macros

// Router
pub mod router;
pub use router::{decode_calls, route, FuncCall};

// Panic handling
mod panic;
pub use panic::vm_panic;

/* --------------------------- Assertion Utilities -------------------------- */

/// Aborts execution if condition is false, printing `msg`.
/// 
/// EDUCATIONAL PURPOSE: This function demonstrates defensive programming in VM environments.
/// In virtual machines, especially those running untrusted code, we need to validate
/// all inputs and assumptions to prevent security vulnerabilities and crashes.
/// 
/// DESIGN PATTERN: This is an "assertion" or "guard clause" pattern. Instead of
/// letting invalid conditions cause undefined behavior later, we fail fast with
/// a clear error message. This makes debugging much easier.
/// 
/// SECURITY CONSIDERATIONS:
/// - Always validate inputs before processing them
/// - Provide clear error messages for debugging
/// - Fail fast to prevent cascading errors
/// - In production VMs, you might want to log these failures for monitoring
/// 
/// USAGE EXAMPLES:
/// - require(data.len() >= 8, b"insufficient data");
/// - require(account.balance >= amount, b"insufficient funds");
/// - require(selector < MAX_SELECTORS, b"invalid selector");
/// 
/// PARAMETERS:
/// - condition: Boolean expression that must be true for execution to continue
/// - msg: Error message to display if condition is false (as bytes for no_std compatibility)
/// 
/// BEHAVIOR: If condition is false, the VM will panic with the provided message
/// and halt execution. This prevents the VM from continuing with invalid state.
pub fn require(condition: bool, msg: &[u8]) {
    if !condition {
        vm_panic(msg);
    }
}
