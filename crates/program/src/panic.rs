//! Panic helper and handler for guest programs.

/// Trap into the host with a panic message.
#[inline(always)]
pub fn vm_panic(msg: &[u8]) -> ! {
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!(
            "li a7, 3", // SYSCALL_PANIC
            "ecall",
            in("a0") msg.as_ptr(),
            in("a1") msg.len(),
            options(noreturn),
        );
    }

    #[cfg(not(target_arch = "riscv32"))]
    {
        panic!(
            "vm_panic: {}",
            core::str::from_utf8(msg).unwrap_or("<non-utf8>")
        );
    }
}

/// Guest panic handler for RISC-V builds (only when guest_handlers enabled).
#[cfg(all(target_arch = "riscv32", feature = "guest_handlers"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let msg_bytes = if let Some(s) = info.message().as_str() {
        s.as_bytes()
    } else {
        b"guest panic"
    };
    vm_panic(msg_bytes);
}
