#[macro_export]
macro_rules! logf_syscall {
    ($fmt_ptr:expr, $fmt_len:expr, $args_ptr:expr, $args_len:expr) => {{
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
    }};
}

#[macro_export]
macro_rules! logf {
    ($fmt:expr) => {{
        const MAX_ARGS: usize = 0;
        let args: [u32; MAX_ARGS] = [];
        let fmt_bytes: &[u8] = $fmt;

        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();

        $crate::logf_syscall!(fmt_ptr, fmt_len, args.as_ptr(), 0usize);
    }};

    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let mut i = 0;
        const MAX_ARGS: usize = 32;
        let args_buf: [u32; MAX_ARGS] = {
            let temp_args = [$($arg as u32),+];
            let mut buffer = [0u32; MAX_ARGS];
            let count = temp_args.len();
            while i < count {
                buffer[i] = temp_args[i];
                i += 1;
            }
            buffer
        };

        let fmt_bytes: &[u8] = $fmt;
        // let mut j = 0;
        // let mut i = 0;

        // while i < fmt_bytes.len() {
        //     if fmt_bytes[i] == b'%' {
        //         i += 1;
        //         if i >= fmt_bytes.len() {
        //             $crate::vm_panic(b"logf: format string ends with bare '%'");
        //         }
        //         match fmt_bytes[i] {
        //             b'd' | b'u' | b'x' => {
        //                 j += 1;
        //             }
        //             b'%' => {} // escape
        //             _ => continue, // ignore unsupported for now
        //         }
        //     }
        //     i += 1;
        // }

        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();
        let args_ptr = args_buf.as_ptr();
        let args_len = i * core::mem::size_of::<u32>();

        $crate::logf_syscall!(fmt_ptr, fmt_len, args_ptr, args_len);
    }};
}

