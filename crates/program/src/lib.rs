#![no_std]  

pub mod pubkey;
pub mod result;
pub use pubkey::Pubkey;
pub use result::Result;

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
