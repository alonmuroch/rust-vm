#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main(a: u32, b: u32) -> u32 {
    let mut result = 0;

    if a > b {
        result = 1;
    } else {
        result = 2;
    }

    result
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
