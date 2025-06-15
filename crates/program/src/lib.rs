#![no_std]  

pub mod pubkey;
pub mod result;
pub use pubkey::Pubkey;
pub use result::Result;

pub mod log;
#[macro_use]
pub use crate::log::*;

pub extern crate hex;
pub extern crate heapless;

#[macro_use] // enables macro use across the crate
pub mod entrypoint;
#[macro_use]
pub mod storage;

// Re-export so `$crate::Persistent` works in the macro
pub use storage::Persistent;

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

pub fn vm_panic(msg: &[u8]) -> ! {
    unsafe {
        // Syscall number 3 = custom "panic with message"
        core::arch::asm!(
            "li a7, 3",
            "ecall",
            in("x10") msg.as_ptr(), // a0 = x10
            in("x11") msg.len(),    // a1 = x11
        );

        // Halt execution explicitly
        core::arch::asm!("ebreak", options(nomem, nostack));
    }

    loop {}
}