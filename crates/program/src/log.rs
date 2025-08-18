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
        // Handle both string literals and byte strings
        let fmt_bytes: &[u8] = $crate::as_bytes!($fmt);
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
        
        let fmt_bytes: &[u8] = $crate::as_bytes!($fmt);
        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();
        let args_ptr = args_buf.as_ptr();
        let args_len = i * core::mem::size_of::<u32>();

        $crate::logf_syscall!(fmt_ptr, fmt_len, args_ptr, args_len);
    }};
}

// Simple string concatenation helper for no_std
#[macro_export]
macro_rules! concat_str {
    ($buf:expr, $($part:expr),+) => {{
        let buf = &mut $buf;
        let mut pos = 0;
        $(
            let part = $part;
            if pos + part.len() <= buf.len() {
                buf[pos..pos + part.len()].copy_from_slice(part);
                pos += part.len();
            }
        )+
        &buf[..pos]
    }};
}

// Alternative: concat with internal buffer (no explicit buffer needed)
#[macro_export]
macro_rules! concat {
    ($($part:expr),+) => {{
        let mut _internal_buf = [0u8; 128]; // Default 128-byte buffer
        $crate::concat_str!(_internal_buf, $($part),+)
    }};
}

// Simplified logging macro that automatically handles arrays/strings
#[macro_export]
macro_rules! log {
    ($fmt:expr) => {
        $crate::logf!($fmt)
    };
    
    // For strings/arrays - automatically pass ptr and len
    ($fmt:expr, $arg:expr) => {{
        let arg = &$arg;
        $crate::logf!($fmt, arg.as_ptr() as u32, arg.len() as u32);
    }};
    
    // For multiple arguments (scalars)
    ($fmt:expr, $($arg:expr),+) => {
        $crate::logf!($fmt, $($arg),+)
    };
}

// Helper macro to handle both string literals and byte strings
#[macro_export]
macro_rules! as_bytes {
    ($s:expr) => {{
        // Use a trait to handle both &str and &[u8]
        trait AsBytes {
            fn as_bytes_ref(&self) -> &[u8];
        }
        
        impl AsBytes for &str {
            fn as_bytes_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }
        
        impl AsBytes for &[u8] {
            fn as_bytes_ref(&self) -> &[u8] {
                self
            }
        }
        
        impl<const N: usize> AsBytes for &[u8; N] {
            fn as_bytes_ref(&self) -> &[u8] {
                *self
            }
        }
        
        let s = &$s;
        s.as_bytes_ref()
    }};
}