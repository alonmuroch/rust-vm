use crate::cpu::CPU;
use crate::memory::Memory;
use storage::Storage;
use crate::registers::Register;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct User {
    pub id: u64,
    pub active: bool,
    pub level: u8,
}

enum Arg {
    U32(u32),
    F32(f32),
    Char(char),
    Str(String),
    Bytes(Vec<u8>),
}

impl CPU {
    pub fn handle_syscall(&mut self, memory: &Memory, storage: &Storage) -> bool {
        let syscall_id = self.regs[Register::A7 as usize]; // a7

        // Arguments from t0–t6
        let args = [
            self.regs[Register::T0 as usize],
            self.regs[Register::T1 as usize],
            self.regs[Register::T2 as usize],
            self.regs[Register::T3 as usize],
            self.regs[Register::T4 as usize],
            self.regs[Register::T5 as usize],
        ];

        let result = match syscall_id {
            1 => self.sys_storage_get(args, memory, storage),
            2 => self.sys_storage_set(args, memory, storage),
            3 => self.sys_panic_with_message(memory),
            4 => self.sys_log(args, memory),
            _ => {
                panic!("Unknown syscall: {}", syscall_id);
            }
        };

        // Return result in t2
        self.regs[Register::T6 as usize] = result;

        true // continue execution
    }

    pub fn sys_storage_get(&mut self, args: [u32; 6], memory: &Memory, storage: &Storage) -> u32 {
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;

        let key_slice = {
            // create a limited scope
            let key_slice_ref = match memory.mem_slice(key_ptr, key_ptr + key_len) {
                Some(r) => r,
                None => return 0,
            };
            key_slice_ref.as_ref().to_vec() // clone to avoid borrow
        };

        let key = match core::str::from_utf8(&key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        println!("🔑 Storage GET key: \"{}\" @ 0x{:08x} (len = {})", key, key_ptr, key_len);

        // Lookup the value in storage
        if let Some(value) = storage.get(key) {
            // alloc and return real address
            let mut buf = (value.len() as u32).to_le_bytes().to_vec();
            buf.extend_from_slice(value.as_slice());
            let addr = memory.alloc_on_heap(&buf);

            println!(
                "📦 Storage GET buffer (total = {}) @ 0x{:08x}: {:02x?}",
                buf.len(),
                addr,
                buf
            );

            return addr;
        } else {
            println!("❌ Key not found in storage");
            0
        }
    }

    pub fn sys_storage_set(&mut self, args: [u32; 6], memory: &Memory, storage: &Storage) -> u32 {
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;
        let val_ptr = args[2] as usize;
        let val_len = args[3] as usize;

        // Safely get the key slice from memory
        let key_slice_ref = match memory.mem_slice(key_ptr, key_ptr + key_len) {
            Some(r) => r,
            None => return 0,
        };
        let key_slice = key_slice_ref.as_ref();

        // Convert the key slice to a &str
        let key: &str = match core::str::from_utf8(key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let value_slice_ref = match memory.mem_slice(val_ptr, val_ptr + val_len) {
            Some(r) => r,
            None => return 0,
        };
        let value_slice = value_slice_ref.as_ref();
        
        println!(
            "🔑 Storage SET key: \"{}\" @ 0x{:08x} (len = {}) | value: {:02x?}",
            key,
            key_ptr,
            key_len,
            value_slice
        );

        storage.set(key, value_slice.to_vec());
        0
    }

    fn sys_panic_with_message(&mut self, memory: &Memory) -> u32 {
        let msg_ptr = self.regs[10] as usize; // a0
        let msg_len = self.regs[11] as usize; // a1

        let msg = memory
            .mem_slice(msg_ptr, msg_ptr + msg_len)
            .map(|bytes| {
                // Convert to String to avoid borrowing temp reference
                String::from_utf8_lossy(&bytes).into_owned()
            })
            .unwrap_or_else(|| "<invalid memory access>".to_string());

        panic!("🔥 Guest panic: {}", msg);
    }

    pub fn sys_log(&mut self, args: [u32; 6], memory: &Memory) -> u32 {
        let [fmt_ptr, fmt_len, arg_ptr, arg_len, ..] = args;

        // 1. Load format string
        let fmt_slice = match memory.mem_slice(fmt_ptr as usize, (fmt_ptr + fmt_len) as usize) {
            Some(s) => s,
            None => {
                println!("⚠️ invalid format string @ 0x{:08x}", fmt_ptr);
                return 0;
            }
        };
        let fmt_bytes = fmt_slice.as_ref();
        let fmt = match core::str::from_utf8(fmt_bytes) {
            Ok(s) => s,
            Err(e) => {
                println!("⚠️ invalid UTF-8 in format string");
                println!("📦 bytes: {:?}", fmt_bytes);
                println!("❌ error: {}", e);
                return 0;
            }
        };

        // 2. Load raw u32 argument buffer
        let args_bytes_slice = memory.mem_slice(arg_ptr as usize, (arg_ptr + arg_len) as usize);
        let args_bytes_holder;
        let args_bytes: &[u8] = if let Some(slice) = args_bytes_slice {
            args_bytes_holder = slice;
            args_bytes_holder.as_ref()
        } else {
            b""
        };

        let raw_args: Vec<u32> = args_bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let mut args: Vec<Arg> = Vec::new();
        let mut raw_iter = raw_args.into_iter();

        let mut chars = fmt.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '%' { continue; }

            let spec: char = chars.next().unwrap_or('%');
            let mut next = || raw_iter.next().unwrap_or(0);

            match spec {
                'd' | 'u' | 'x' => args.push(Arg::U32(next())),
                'f' => args.push(Arg::F32(f32::from_bits(next()))),
                'c' => args.push(Arg::Char(char::from_u32(next()).unwrap_or('?'))),

                's' => {
                    let ptr = next() as usize;
                    let len = next() as usize;
                    match memory.mem_slice(ptr, ptr + len) {
                        Some(slice) => {
                            let s_ptr = core::str::from_utf8(slice.as_ref());
                            args.push(match s_ptr {
                                Ok(s) => Arg::Str(s.to_string()),
                                Err(_) => Arg::Str("<invalid>".to_string()),
                            });
                        }
                        None => {
                            args.push(Arg::Str("<invalid>".to_string()));
                        }
                    }

                }

                'b' => {
                    let ptr = next() as usize;
                    let len = next() as usize;
                    match memory.mem_slice(ptr, ptr + len) {
                        Some(slice) => {
                            args.push(Arg::Bytes(slice.to_vec()));
                        }
                        None => {
                            args.push(Arg::Str("<invalid>".to_string()));
                        }
                    }
                }

                _ => args.push(Arg::Str("<bad-format>".to_string())),
            }
        }

        // 3. Render log output
        let mut output = String::new();
        let mut args_iter = args.iter();
        let mut fmt_chars = fmt.chars().peekable();

        while let Some(c) = fmt_chars.next() {
            if c == '%' {
                match fmt_chars.next() {
                    Some('d') | Some('u') => match args_iter.next() {
                        Some(Arg::U32(v)) => output.push_str(&format!("{}", *v as i32)),
                        _ => output.push_str("<err>"),
                    },
                    Some('x') => match args_iter.next() {
                        Some(Arg::U32(v)) => output.push_str(&format!("{:08x}", v)),
                        _ => output.push_str("<err>"),
                    },
                    Some('f') => match args_iter.next() {
                        Some(Arg::F32(f)) => output.push_str(&format!("{}", f)),
                        _ => output.push_str("<err>"),
                    },
                    Some('c') => match args_iter.next() {
                        Some(Arg::Char(c)) => output.push(*c),
                        _ => output.push_str("<err>"),
                    },
                    Some('s') => match args_iter.next() {
                        Some(Arg::Str(s)) => output.push_str(s),
                        _ => output.push_str("<err>"),
                    },
                    Some('b') => match args_iter.next() {
                        Some(Arg::Bytes(b)) => output.push_str(&format!("{:?}", b)),
                        _ => output.push_str("<err>"),
                    },
                    Some('%') => output.push('%'),
                    Some(_) | None => output.push_str("<%?>"),
                }
            } else {
                output.push(c);
            }
        }

        println!("📜 Guest Log: {}", output);
        0
    }


}
