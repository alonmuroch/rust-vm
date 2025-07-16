use crate::cpu::CPU;
use crate::memory_page::MemoryPage;
use storage::Storage;
use crate::registers::Register;
use std::rc::Rc;
use core::cell::RefCell;
use crate::host_interface::HostInterface;
use hex;

/// Example user data structure for demonstration purposes.
/// 
/// EDUCATIONAL: This shows how you might represent user data in a VM.
/// In real systems, this could contain user permissions, session data,
/// or other user-specific information.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct User {
    pub id: u64,        // Unique user identifier
    pub active: bool,   // Whether the user is currently active
    pub level: u8,      // User privilege level (0 = normal, 1 = admin, etc.)
}



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

impl CPU {
    /// Handles system calls from user programs.
    /// 
    /// EDUCATIONAL PURPOSE: This function implements the system call interface
    /// between user programs and the VM. System calls allow user programs to
    /// request services from the VM (like storage access, logging, etc.).
    /// 
    /// RISC-V SYSTEM CALL CONVENTION:
    /// - System call ID is passed in register A7
    /// - Arguments are passed in registers A1-A6
    /// - Return value is placed in register A0
    /// 
    /// SYSTEM CALL TYPES:
    /// - 1: Storage GET - retrieve data from persistent storage
    /// - 2: Storage SET - store data in persistent storage
    /// - 3: Panic - terminate execution with a message
    /// - 4: Log - print formatted output
    /// 
    /// SECURITY: System calls provide a controlled interface between user
    /// programs and system resources, preventing direct access to sensitive data.
    pub fn handle_syscall(
        &mut self, 
        memory: Rc<RefCell<MemoryPage>>, 
        storage: Rc<RefCell<Storage>>,
        host: &mut Box<dyn HostInterface>) -> bool {
        // EDUCATIONAL: Get the system call ID from register a7
        let syscall_id = self.regs[Register::A7 as usize]; // a7

        // EDUCATIONAL: Extract arguments from registers t0-t5
        // This follows the RISC-V calling convention for system calls
        let args = [
            self.regs[Register::A1 as usize],
            self.regs[Register::A2 as usize],
            self.regs[Register::A3 as usize],
            self.regs[Register::A4 as usize],
            self.regs[Register::A5 as usize],
            self.regs[Register::A6 as usize],
        ];

        // EDUCATIONAL: Dispatch to the appropriate system call handler
        let result = match syscall_id {
            SYSCALL_STORAGE_GET => self.sys_storage_get(args, memory, storage),
            SYSCALL_STORAGE_SET => self.sys_storage_set(args, memory, storage),
            SYSCALL_PANIC => self.sys_panic_with_message(memory),
            SYSCALL_LOG => self.sys_log(args, memory),
            SYSCALL_CALL_PROGRAM => self.sys_call_program(args, memory, host),
            SYSCALL_FIRE_EVENT => self.sys_fire_event(args, memory, host),
            _ => {
                // EDUCATIONAL: Unknown system calls should panic
                panic!("Unknown syscall: {}", syscall_id);
            }
        };

        // EDUCATIONAL: Store the result in register t6 for the user program
        self.regs[Register::A0 as usize] = result;

        true // continue execution
    }

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

    pub fn sys_call_program(
        &mut self,
        args: [u32; 6],
        memory: Rc<RefCell<MemoryPage>>,
        host: &mut Box<dyn HostInterface>,
    ) -> u32 {
        let to_ptr = args[0] as usize;
        let from_ptr = args[1] as usize;
        let input_ptr = args[2] as usize;
        let input_len = args[3] as usize;

        let result_ptr: u32;
        let page_index: usize;
        {
            let borrowed_memory = memory.borrow();

            // === Read guest memory ===
            let to_slice = match borrowed_memory.mem_slice(to_ptr, to_ptr + 20) {
                Some(r) => r,
                None => return 0, // Error: invalid to_ptr
            };
            let from_slice = match borrowed_memory.mem_slice(from_ptr, from_ptr + 20) {
                Some(r) => r,
                None => return 0, // Error: invalid from_ptr
            };
            let input_slice = match borrowed_memory.mem_slice(input_ptr, input_ptr + input_len) {
                Some(r) => r,
                None => return 0, // Error: invalid input_ptr
            };

            // === Copy slices into fixed and owned types ===
            let mut to_bytes = [0u8; 20];
            let mut from_bytes = [0u8; 20];
            to_bytes.copy_from_slice(&to_slice);
            from_bytes.copy_from_slice(&from_slice);
            let input_vec = input_slice.to_vec();

            // === Call the host ===
            (result_ptr, page_index) = host.call_program(from_bytes, to_bytes, input_vec);
        }
        
        {
            let borrowed_memory = memory.borrow_mut();
            // === Read result from callee’s memory via host ===
            let result_bytes = match host.read_memory_page(page_index, result_ptr, 5) {
                Some(b) => b,
                None => return 0, // Failed to read result
            };

            // === Allocate heap memory in caller's memory page ===
            borrowed_memory.alloc_on_heap(&result_bytes)
        }
    }

    /// System call to retrieve data from persistent storage.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how to implement storage access
    /// in a VM. Storage provides persistent data that survives between
    /// program executions.
    /// 
    /// PARAMETERS (from args array):
    /// - args[0]: Pointer to key string in memory
    /// - args[1]: Length of key string
    /// 
    /// RETURN VALUE: 
    /// - 0: Key not found
    /// - Non-zero: Pointer to result buffer containing [length][data]
    /// 
    /// MEMORY SAFETY: Validates all memory accesses to prevent crashes
    /// and ensure the VM remains stable.
    pub fn sys_storage_get(&mut self, args: [u32; 6], memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> u32 {
        // EDUCATIONAL: Extract key pointer and length from arguments
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;

        let borrowed_memory = memory.borrow();

        // EDUCATIONAL: Safely read the key from memory
        let key_slice = {
            // EDUCATIONAL: Create a limited scope to avoid borrow checker issues
            let key_slice_ref = match borrowed_memory.mem_slice(key_ptr, key_ptr + key_len) {
                Some(r) => r,
                None => return 0,  // Invalid memory access
            };
            key_slice_ref.as_ref().to_vec() // clone to avoid borrow
        };

        // EDUCATIONAL: Convert bytes to string, handling invalid UTF-8
        let key: String = key_slice
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() {
                    (b as char).to_string()
                } else {
                    format!("{:02x}", b)
                }
            })
            .collect();

        println!("🔑 Storage GET key: \"{}\" @ 0x{:08x} (len = {})", key, key_ptr, key_len);

        // EDUCATIONAL: Look up the value in storage
        if let Some(value) = storage.borrow().get(&key) {
            // EDUCATIONAL: Create result buffer with format [length][data]
            let mut buf = (value.len() as u32).to_le_bytes().to_vec();
            buf.extend_from_slice(value.as_slice());
            
            // EDUCATIONAL: Allocate memory for the result and return its address
            let addr = borrowed_memory.alloc_on_heap(&buf);

            println!(
                "📦 Storage GET buffer (total = {}) @ 0x{:08x}: {:02x?}",
                buf.len(),
                addr,
                buf
            );

            return addr;
        } else {
            println!("❌ Key not found in storage");
            0  // Key not found
        }
    }

    /// System call to store data in persistent storage.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how to implement persistent
    /// storage in a VM. Data stored here survives between program runs.
    /// 
    /// PARAMETERS (from args array):
    /// - args[0]: Pointer to key string in memory
    /// - args[1]: Length of key string
    /// - args[2]: Pointer to value data in memory
    /// - args[3]: Length of value data
    /// 
    /// RETURN VALUE: Always 0 (success)
    /// 
    /// PERSISTENCE: Data stored here is saved to the VM's persistent storage
    /// and can be retrieved later using sys_storage_get.
    pub fn sys_storage_set(&mut self, args: [u32; 6], memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> u32 {
        // EDUCATIONAL: Extract key and value pointers/lengths from arguments
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;
        let val_ptr = args[2] as usize;
        let val_len = args[3] as usize;

        let borrowed_memory = memory.borrow();

        // EDUCATIONAL: Safely get the key slice from memory
        let key_slice_ref = match borrowed_memory.mem_slice(key_ptr, key_ptr + key_len) {
            Some(r) => r,
            None => panic!(
                "invalid key memory access: key_ptr=0x{:08x}, key_len={}",
                key_ptr, key_len
            ),  // Invalid memory access
        };
        let key_slice = key_slice_ref.as_ref();

        // EDUCATIONAL: Convert the key slice to a string
        let key: String = key_slice
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() {
                    (b as char).to_string()
                } else {
                    format!("{:02x}", b)
                }
            })
            .collect();

        // EDUCATIONAL: Safely get the value slice from memory
        let value_slice_ref = match borrowed_memory.mem_slice(val_ptr, val_ptr + val_len) {
            Some(r) => r,
            None => panic!("invalid value memory access"),  // Invalid memory access
        };
        let value_slice = value_slice_ref.as_ref();
        
        println!(
            "🔑 Storage SET key: \"{}\" @ 0x{:08x} (len = {}) | value: {:02x?}",
            key,
            key_ptr,
            key_len,
            value_slice
        );

        // EDUCATIONAL: Store the key-value pair in persistent storage
        storage.borrow_mut().set(&key, value_slice.to_vec());
        0  // Success
    }

    /// System call to panic with a custom message.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how to handle program termination
    /// in a controlled way. Panics allow programs to terminate with an error
    /// message rather than just crashing.
    /// 
    /// PARAMETERS (from registers):
    /// - a0: Pointer to panic message in memory
    /// - a1: Length of panic message
    /// 
    /// BEHAVIOR: Terminates the entire VM execution with the provided message.
    /// This is useful for debugging and error reporting.
    fn sys_panic_with_message(&mut self, memory: Rc<RefCell<MemoryPage>>) -> u32 {
        // EDUCATIONAL: Get message pointer and length from registers
        let msg_ptr = self.regs[10] as usize; // a0
        let msg_len = self.regs[11] as usize; // a1

        // EDUCATIONAL: Safely read the panic message from memory
        let msg = memory
            .borrow()
            .mem_slice(msg_ptr, msg_ptr + msg_len)
            .map(|bytes| {
                // EDUCATIONAL: Convert to String to avoid borrowing temp reference
                String::from_utf8_lossy(&bytes).into_owned()
            })
            .unwrap_or_else(|| "<invalid memory access>".to_string());

        // EDUCATIONAL: Panic with the user-provided message
        panic!("🔥 Guest panic: {}", msg);
    }

    /// System call to print formatted output (like printf).
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how to implement formatted output
    /// in a VM. This is essential for debugging and user interaction.
    /// 
    /// PARAMETERS (from args array):
    /// - args[0]: Pointer to format string in memory
    /// - args[1]: Length of format string
    /// - args[2]: Pointer to argument buffer in memory
    /// - args[3]: Length of argument buffer
    /// 
    /// FORMAT STRING SUPPORT: Supports basic format specifiers like %d, %u, %x, %f, %c, %s, %b
    /// 
    /// RETURN VALUE: Always 0 (success)
    pub fn sys_log(&mut self, args: [u32; 6], memory: Rc<RefCell<MemoryPage>>) -> u32 {
        let [fmt_ptr, fmt_len, arg_ptr, arg_len, ..] = args;

        let borrowed_memory = memory.borrow();

        // EDUCATIONAL: Step 1 - Load and validate the format string
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

        // EDUCATIONAL: Step 2 - Load the raw argument buffer
        let args_bytes_slice = borrowed_memory.mem_slice(arg_ptr as usize, (arg_ptr + arg_len) as usize);
        let args_bytes_holder;
        let args_bytes: &[u8] = if let Some(slice) = args_bytes_slice {
            args_bytes_holder = slice;
            args_bytes_holder.as_ref()
        } else {
            b""
        };

        // EDUCATIONAL: Convert raw bytes to u32 arguments
        let raw_args: Vec<u32> = args_bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // EDUCATIONAL: Step 3 - Parse format string and extract arguments
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
