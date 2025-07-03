#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack, preserves_flags));
    }
    loop {}
}

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

#[cfg(not(target_arch = "riscv32"))]
pub fn vm_panic(msg: &[u8]) -> ! {
    panic!("vm_panic: {}", core::str::from_utf8(msg).unwrap_or("<invalid utf-8>"));
}
