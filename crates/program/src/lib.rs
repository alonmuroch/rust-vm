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
pub fn require(condition: bool, msg: &[u8]) {
    if !condition {
        vm_panic(msg);
    }
}
