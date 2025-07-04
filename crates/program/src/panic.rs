use core::fmt::Write;

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
    let mut buf = [0u8; 128];

    struct BufWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl<'a> Write for BufWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let space = self.buf.len().saturating_sub(self.pos);
            let len = core::cmp::min(s.len(), space);
            self.buf[self.pos..self.pos + len].copy_from_slice(&s.as_bytes()[..len]);
            self.pos += len;
            Ok(())
        }
    }

    let mut writer = BufWriter { buf: &mut buf, pos: 0 };

    if let Some(location) = info.location() {
        let _ = write!(
            &mut writer,
            "panic at {}:{}:{}: ",
            location.file(),
            location.line(),
            location.column()
        );
    }

    let _ = write!(&mut writer, "{}", info.message());

    vm_panic(&writer.buf[..writer.pos]);
}

#[cfg(not(target_arch = "riscv32"))]
pub fn vm_panic(msg: &[u8]) -> ! {
    panic!("vm_panic: {}", core::str::from_utf8(msg).unwrap_or("<invalid utf-8>"));
}
