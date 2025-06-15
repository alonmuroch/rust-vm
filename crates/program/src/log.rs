/// Log message using syscall ID 4.
/// - a0 = ptr to message
/// - a1 = length of message
/// - a7 = 4 (syscall ID)
pub fn vm_log(msg: &[u8]) {
    unsafe {
        core::arch::asm!(
            "li a7, 4",         // syscall ID 4 = log
            "ecall",            // make syscall
            in("a0") msg.as_ptr(),
            in("a1") msg.len(),
        );
    }
}

/// Log a string message from guest to host (uses syscall 4)
pub fn vm_log_str(msg: &str) {
    vm_log(msg.as_bytes());
}
#[macro_export]
macro_rules! logf {
    ($($arg:tt)*) => {{
        use $crate::heapless::String; // ✅ macro hygiene: no need to depend on heapless in downstream crates
        use core::fmt::Write;

        let mut s: String<128> = String::new();
        let _ = write!(&mut s, $($arg)*);
        $crate::log::vm_log_str(&s);
    }};
}
