#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Mark the symbol as externally visible and entry point
#[no_mangle]
pub extern "C" fn main() -> i32 {
    5 + 10
}
