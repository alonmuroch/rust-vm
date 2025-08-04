use crate::cpu::CPU;
use crate::memory_page::MemoryPage;
use storage::Storage;
use crate::registers::Register;
use std::rc::Rc;
use core::cell::RefCell;
use crate::host_interface::HostInterface;
use std::any::Any;

/// System call IDs for the VM.
pub const SYSCALL_STORAGE_GET: u32 = 1;
pub const SYSCALL_STORAGE_SET: u32 = 2;
pub const SYSCALL_PANIC: u32 = 3;
pub const SYSCALL_LOG: u32 = 4;
pub const SYSCALL_CALL_PROGRAM: u32 = 5;
pub const SYSCALL_FIRE_EVENT: u32 = 6;
/// Represents different types of arguments that can be passed to system calls.
/// 
/// EDUCATIONAL: This enum demonstrates how to handle different data types
/// in system calls. In real operating systems, system calls need to handle
/// various data types safely.
enum Arg {
    U32(u32),           // 32-bit unsigned integer
    F32(f32),           // 32-bit floating point
    Char(char),         // Single character
    Str(String),        // String (owned)
    Bytes(Vec<u8>),     // Raw bytes
}

pub trait SyscallHandler: std::fmt::Debug {
    fn handle_syscall(
        &mut self,
        call_id: u32,
        args: [u32; 5],
        memory: Rc<RefCell<MemoryPage>>,
        storage: Rc<RefCell<Storage>>,
        host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
    ) -> (u32, bool);
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct DefaultSyscallHandler;

impl DefaultSyscallHandler {
	pub fn sys_fire_event(&mut self, args: [u32; 6], memory: Rc<RefCell<MemoryPage>>, host: &mut Box<dyn HostInterface>,) -> u32 {
        // EDUCATIONAL: Extract key pointer and length from arguments
        let ptr = args[0] as usize;
        let len = args[1] as usize;

        let borrowed_memory = memory.borrow();

        // EDUCATIONAL: Safely read the key from memory
        // EDUCATIONAL: Create a limited scope to avoid borrow checker issues
        let event_bytes = match borrowed_memory.mem_slice(ptr, ptr + len) {
            Some(r) => r,
            None => panic!("invalid memory access"),  // Invalid memory access
        };

        host.fire_event(event_bytes.to_vec());
        0
    }

    fn sys_storage_get(&mut self, args: [u32; 5], memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> u32 {
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;
        let borrowed_memory = memory.borrow();
        let key_slice = {
            let key_slice_ref = match borrowed_memory.mem_slice(key_ptr, key_ptr + key_len) {
                Some(r) => r,
                None => return 0,
            };
            key_slice_ref.as_ref().to_vec()
        };
        let key = match core::str::from_utf8(&key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        if let Some(value) = storage.borrow().get(key) {
            let mut buf = (value.len() as u32).to_le_bytes().to_vec();
            buf.extend_from_slice(value.as_slice());
            let addr = borrowed_memory.alloc_on_heap(&buf);
            return addr;
        } else {
            0
        }
    }

    fn sys_storage_set(&mut self, args: [u32; 5], memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> u32 {
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;
        let val_ptr = args[2] as usize;
        let val_len = args[3] as usize;
        let borrowed_memory = memory.borrow();
        let key_slice_ref = match borrowed_memory.mem_slice(key_ptr, key_ptr + key_len) {
            Some(r) => r,
            None => return 0,
        };
        let key_slice = key_slice_ref.as_ref();
        let key: &str = match core::str::from_utf8(key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let value_slice_ref = match borrowed_memory.mem_slice(val_ptr, val_ptr + val_len) {
            Some(r) => r,
            None => return 0,
        };
        let value_slice = value_slice_ref.as_ref();
        storage.borrow_mut().set(key, value_slice.to_vec());
        0
    }

    fn sys_panic_with_message(&mut self, regs: &mut [u32; 32], memory: Rc<RefCell<MemoryPage>>) -> u32 {
        let msg_ptr = regs[Register::A0 as usize] as usize;
        let msg_len = regs[Register::A1 as usize] as usize;
        let msg = memory
            .borrow()
            .mem_slice(msg_ptr, msg_ptr + msg_len)
            .map(|bytes| {
                String::from_utf8_lossy(&bytes).into_owned()
            })
            .unwrap_or_else(|| "<invalid memory access>".to_string());
        panic!("🔥 Guest panic: {}", msg);
    }

    fn sys_log(&mut self, args: [u32; 5], memory: Rc<RefCell<MemoryPage>>) -> u32 {
        let [fmt_ptr, fmt_len, arg_ptr, arg_len, ..] = args;
        let borrowed_memory = memory.borrow();
        let fmt_slice = match borrowed_memory.mem_slice(fmt_ptr as usize, (fmt_ptr + fmt_len) as usize) {
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
        let args_bytes_slice = borrowed_memory.mem_slice(arg_ptr as usize, (arg_ptr + arg_len) as usize);
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
                    match borrowed_memory.mem_slice(ptr, ptr + len) {
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
                    match borrowed_memory.mem_slice(ptr, ptr + len) {
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

    fn sys_call_program(&mut self, args: [u32; 5], memory: Rc<RefCell<MemoryPage>>, host: &mut Box<dyn HostInterface>) -> u32 {
        let to_ptr = args[0] as usize;
        let from_ptr = args[1] as usize;
        let input_ptr = args[2] as usize;
        let input_len = args[3] as usize;
        let result_ptr: u32;
        let page_index: usize;
        {
            let borrowed_memory = memory.borrow();
            let to_slice = match borrowed_memory.mem_slice(to_ptr, to_ptr + 20) {
                Some(r) => r,
                None => return 0,
            };
            let from_slice = match borrowed_memory.mem_slice(from_ptr, from_ptr + 20) {
                Some(r) => r,
                None => return 0,
            };
            let input_slice = match borrowed_memory.mem_slice(input_ptr, input_ptr + input_len) {
                Some(r) => r,
                None => return 0,
            };
            let mut to_bytes = [0u8; 20];
            let mut from_bytes = [0u8; 20];
            to_bytes.copy_from_slice(&to_slice);
            from_bytes.copy_from_slice(&from_slice);
            let input_vec = input_slice.to_vec();
            (result_ptr, page_index) = host.call_program(from_bytes, to_bytes, input_vec);
        }
        {
            let borrowed_memory = memory.borrow_mut();
            let result_bytes = match host.read_memory_page(page_index, result_ptr, 5) {
                Some(b) => b,
                None => return 0,
            };
            borrowed_memory.alloc_on_heap(&result_bytes)
        }
    }
}

impl SyscallHandler for DefaultSyscallHandler {
    fn handle_syscall(
        &mut self,
        call_id: u32,
        args: [u32; 5],
        memory: Rc<RefCell<MemoryPage>>,
        storage: Rc<RefCell<Storage>>,
        host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
    ) -> (u32, bool) {
        let result = match call_id {
            SYSCALL_STORAGE_GET => self.sys_storage_get(args, memory, storage),
            SYSCALL_STORAGE_SET => self.sys_storage_set(args, memory, storage),
            SYSCALL_PANIC => self.sys_panic_with_message(regs, memory),
            SYSCALL_LOG => self.sys_log(args, memory),
            SYSCALL_CALL_PROGRAM => self.sys_call_program(args, memory, host),
            _ => {
                panic!("Unknown syscall: {}", call_id);
            }
        };
        (result, true)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
