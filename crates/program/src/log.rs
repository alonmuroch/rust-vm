#[macro_export]
macro_rules! logf_syscall {
    ($fmt_ptr:expr, $fmt_len:expr, $args_ptr:expr, $args_len:expr) => {{
        unsafe {
            core::arch::asm!(
                "li a7, 4",  // syscall_storage_read
                "ecall",
                in("t0") $fmt_ptr,
                in("t1") $fmt_len,
                in("t2") $args_ptr,
                in("t3") $args_len,
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
        const MAX_ARGS: usize = 32;
        let mut args: [u32; MAX_ARGS] = [0; MAX_ARGS];
        let mut j = 0;

        let mut arg_iter = [$($arg),+].into_iter();
        let fmt_bytes: &[u8] = $fmt;
        let mut i = 0;

        while i < fmt_bytes.len() {
            if fmt_bytes[i] == b'%' {
                i += 1;
                if i >= fmt_bytes.len() {
                    $crate::vm_panic(b"logf: format string ends with bare '%'");
                }

                match fmt_bytes[i] {
                    b'd' | b'u' | b'x' => {
                        let val = arg_iter.next().unwrap_or_else(|| {
                            $crate::vm_panic(b"logf: missing integer argument");
                        });
                        args[j] = val;
                        j += 1;
                    }
                    // b'f' => {
                    //     let val = arg_iter.next().unwrap_or_else(|| {
                    //         $crate::vm_panic(b"logf: missing float argument");
                    //     });
                    //     args[j] = (val as f32).to_bits();
                    //     j += 1;
                    // }
                    // b'c' => {
                    //     let val = arg_iter.next().unwrap_or_else(|| {
                    //         $crate::vm_panic(b"logf: missing char argument");
                    //     });
                    //     args[j] = val as u32;
                    //     j += 1;
                    // }
                    // b'b' => {
                    //     let ptr = arg_iter.next().unwrap_or_else(|| {
                    //         $crate::vm_panic(b"logf: missing slice ptr");
                    //     });
                    //     let len = arg_iter.next().unwrap_or_else(|| {
                    //         $crate::vm_panic(b"logf: missing slice len");
                    //     });
                    //     args[j] = ptr;
                    //     args[j + 1] = len;
                    //     j += 2;
                    // }
                    _ => $crate::vm_panic(b"logf: unknown format specifier"),
                }
            }
            i += 1;
        }

        let fmt_ptr = fmt_bytes.as_ptr();
        let fmt_len = fmt_bytes.len();
        let args_ptr = args.as_ptr();
        let args_len = j * core::mem::size_of::<u32>();

        $crate::logf_syscall!(fmt_ptr, fmt_len, args_ptr, args_len);
    }};
}

