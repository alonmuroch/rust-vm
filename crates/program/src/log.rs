/// Sends a format string and packed arguments to the host for log formatting.
/// - `fmt`: The format string, e.g. `"key = %s, id = %x"`
/// - `args`: Flat list of `u32` values, derived from the format string (e.g. integers, ptr/len pairs)
///
/// Internally triggers syscall ID 5:
/// - a0 = fmt_ptr
/// - a1 = fmt_len
/// - a2 = args_ptr
/// - a3 = args_len (in bytes)
/// - a7 = 5 (syscall ID)
pub fn vm_logf(fmt: &[u8], args: &[u32]) {
    let fmt_ptr = fmt.as_ptr();
    let fmt_len = fmt.len();
    let args_ptr = args.as_ptr() as *const u8;
    let args_len = args.len() * core::mem::size_of::<u32>();

    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") fmt_ptr,
            in("a1") fmt_len,
            in("a2") args_ptr,
            in("a3") args_len,
            in("a7") 4,
        );
    }
}


/// Logs a formatted message using syscall 4 with host-side formatting.
///
/// Format string must be a UTF-8 byte string literal (`b"..."`), and supports:
/// - `%d`: signed/unsigned integer (`i32` / `u32`)
/// - `%u`: unsigned integer (`u32`)
/// - `%x`: hexadecimal (`u32`)
///
/// The macro encodes arguments into a flat `u32` buffer and passes it via syscall:
/// - a0 = format string pointer
/// - a1 = format string length
/// - a2 = pointer to `u32` arg buffer
/// - a3 = size of arg buffer in bytes
/// - a7 = 4 (syscall ID)
///
/// # Example
/// ```
/// let key_len: u32 = 42;
/// logf!(b"loading key of length=%d", key_len);
/// ```
#[macro_export]
macro_rules! logf {
    ($fmt:expr) => {{
        $crate::log::vm_logf($fmt, &[]);
    }};

    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        use core::mem;

        const MAX_ARGS: usize = 32;
        let mut args: [u32; MAX_ARGS] = [0; MAX_ARGS];
        let mut i = 0;

        let mut arg_iter = [$($arg),+].into_iter();
        let fmt_bytes: &[u8] = $fmt;

        let fmt_str = match core::str::from_utf8(fmt_bytes) {
            Ok(s) => s,
            Err(_) => $crate::vm_panic(b"logf: format string must be valid UTF-8"),
        };

        let mut fmt_chars = fmt_str.chars().peekable();

        while let Some(c) = fmt_chars.next() {
            if c != '%' { continue; }
            let spec = match fmt_chars.next() {
                Some(ch) => ch,
                None => $crate::vm_panic(b"logf: format string ends with bare '%'"),
            };

            match spec {
                'd' | 'u' | 'x' => {
                    match arg_iter.next() {
                        Some(val) => {
                            args[i] = val;
                            i += 1;
                        }
                        None => $crate::vm_panic(b"logf: missing integer arg"),
                    }
                }

                // Uncomment and extend these as needed:
                // 'f' => { ... }
                // 'c' => { ... }
                // 's' => { ... }
                // 'b' => { ... }

                _ => $crate::vm_panic(b"logf: unknown format specifier"),
            }
        }

        $crate::log::vm_logf(fmt_bytes, &args[..i]);
    }};
}




