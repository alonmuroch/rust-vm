#[macro_export]
macro_rules! logf_syscall {
    ($fmt_ptr:expr, $fmt_len:expr, $args_ptr:expr, $args_len:expr) => {{
        #[cfg(target_arch = "riscv32")]
        unsafe {
            core::arch::asm!(
                "li a7, 4",  // syscall_log
                "ecall",
                in("a1") $fmt_ptr,
                in("a2") $fmt_len,
                in("a3") $args_ptr,
                in("a4") $args_len,
            );
        }
        #[cfg(not(target_arch = "riscv32"))]
        {
            // For non-RISC-V targets, do nothing
        }
    }};
}

#[macro_export]
macro_rules! logf {
    ($fmt:expr) => {{
        let fmt_bytes: &[u8] = $fmt;
        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();
        $crate::logf_syscall!(fmt_ptr, fmt_len, 0 as *const u32, 0usize);
    }};

    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        // Simple approach: just pass raw values as u32s
        // The host will interpret them based on format specifiers
        const MAX_ARGS: usize = 32;
        let mut args_buf = [0u32; MAX_ARGS];
        let mut i = 0;
        
        $(
            args_buf[i] = $arg as u32;
            i += 1;
            if i >= MAX_ARGS { 
                i = MAX_ARGS - 1; 
            }
        )+
        
        let fmt_bytes: &[u8] = $fmt;
        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();
        let args_ptr = args_buf.as_ptr();
        let args_len = i * core::mem::size_of::<u32>();

        $crate::logf_syscall!(fmt_ptr, fmt_len, args_ptr, args_len);
    }};
}

// Helper macro for arrays - passes pointer and length
#[macro_export]
macro_rules! log_array {
    ($fmt:expr, $arr:expr) => {{
        let arr_ref: &[_] = $arr;
        let ptr = arr_ref.as_ptr() as u32;
        let len = arr_ref.len() as u32;
        $crate::logf!($fmt, ptr, len);
    }};
}