#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let a: u32 = 5;
    let b: u32 = 3;

    let mut result = 0;

    if a > b {
        result = 1;
    } else {
        result = 2;
    }

    // prevent optimization
    unsafe { core::ptr::write_volatile(&mut result as *mut u32, result); }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
