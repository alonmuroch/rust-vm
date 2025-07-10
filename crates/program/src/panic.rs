use crate::logf;

#[cfg(target_arch = "riscv32")]
pub fn vm_panic(msg: &[u8]) -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 3",         // syscall: panic
            "ecall",
            in("a0") msg.as_ptr(),
            in("a1") msg.len(),
        );
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
    loop {}
}

#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(l) = info.location() {
        logf!(l.file().as_bytes());
        logf!(b"panic occurred at line %d", l.line());
    }

    if let Some(s) = info.message().as_str() {
       vm_panic(s.as_bytes());
    } else {    
        vm_panic(b"panic occurred (non-str message)");
    }
}

#[cfg(not(target_arch = "riscv32"))]
pub fn vm_panic(msg: &[u8]) -> ! {
    panic!("vm_panic: {}", core::str::from_utf8(msg).unwrap_or("<invalid utf-8>"));
}
