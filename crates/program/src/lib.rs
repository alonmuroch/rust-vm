#![no_std]

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
#[macro_use]
pub use crate::log::*;

// Entrypoint macro
#[macro_use]
pub mod entrypoint;

// Persistent storage system
#[macro_use]
pub mod storage;
pub use storage::Persistent; // Allow `$crate::Persistent` in macros

/* ----------------------------- Panic Handlers ----------------------------- */

/// Default panic handler for the guest program.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!(
            "ebreak",
            options(nomem, nostack, preserves_flags)
        );
    }
    loop {}
}

/// Triggers a custom VM panic with a message (via syscall).
pub fn vm_panic(msg: &[u8]) -> ! {
    unsafe {
        // Syscall 3 = panic with message
        core::arch::asm!(
            "li a7, 3",           // syscall number
            "ecall",              // trigger it
            in("t0") msg.as_ptr(), 
            in("t1") msg.len(),
        );

        core::arch::asm!("ebreak", options(nomem, nostack));
    }
    loop {}
}

/* --------------------------- Assertion Utilities -------------------------- */

/// Aborts execution if condition is false, printing `msg`.
pub fn require(condition: bool, msg: &[u8]) {
    if !condition {
        vm_panic(msg);
    }
}
