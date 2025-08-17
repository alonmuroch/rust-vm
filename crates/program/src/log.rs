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

// Helper macro that handles array arguments
#[macro_export]
macro_rules! logf_impl {
    // Base case: format string only
    (@parse $fmt:expr, $args:expr, $idx:expr, []) => {};
    
    // Array format specifiers - automatically extract ptr and len
    (@parse $fmt:expr, $args:expr, $idx:expr, [%a $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        let arr = &$arg;
        $args[$idx] = arr.as_ptr() as u32;
        $args[$idx + 1] = arr.len() as u32;
        logf_impl!(@parse $fmt, $args, $idx + 2, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%A $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        let arr = &$arg;
        $args[$idx] = arr.as_ptr() as u32;
        $args[$idx + 1] = arr.len() as u32;
        logf_impl!(@parse $fmt, $args, $idx + 2, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%b $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        let arr = &$arg;
        $args[$idx] = arr.as_ptr() as u32;
        $args[$idx + 1] = arr.len() as u32;
        logf_impl!(@parse $fmt, $args, $idx + 2, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%s $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        let s = &$arg;
        $args[$idx] = s.as_ptr() as u32;
        $args[$idx + 1] = s.len() as u32;
        logf_impl!(@parse $fmt, $args, $idx + 2, [$($rest)*] $(, $more)*);
    }};
    
    // Scalar format specifiers
    (@parse $fmt:expr, $args:expr, $idx:expr, [%d $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        $args[$idx] = $arg as u32;
        logf_impl!(@parse $fmt, $args, $idx + 1, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%u $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        $args[$idx] = $arg as u32;
        logf_impl!(@parse $fmt, $args, $idx + 1, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%x $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        $args[$idx] = $arg as u32;
        logf_impl!(@parse $fmt, $args, $idx + 1, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%c $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        $args[$idx] = $arg as u32;
        logf_impl!(@parse $fmt, $args, $idx + 1, [$($rest)*] $(, $more)*);
    }};
    
    (@parse $fmt:expr, $args:expr, $idx:expr, [%f $($rest:tt)*], $arg:expr $(, $more:expr)*) => {{
        let f: f32 = $arg;
        $args[$idx] = f.to_bits();
        logf_impl!(@parse $fmt, $args, $idx + 1, [$($rest)*] $(, $more)*);
    }};
    
    // Skip non-format characters
    (@parse $fmt:expr, $args:expr, $idx:expr, [$other:tt $($rest:tt)*] $(, $arg:expr)*) => {
        logf_impl!(@parse $fmt, $args, $idx, [$($rest)*] $(, $arg)*);
    };
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
        // Simple fallback - just pass everything as u32s and let host figure it out
        // For arrays, you still need to pass ptr and len manually
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

// Convenience macro for arrays that automatically handles ptr/len
#[macro_export]
macro_rules! logf_array {
    ($fmt:expr, $arr:expr) => {{
        let arr = &$arr;
        $crate::logf!($fmt, arr.as_ptr() as u32, arr.len() as u32);
    }};
}