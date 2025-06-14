use crate::cpu::CPU;
use crate::memory::Memory;
use crate::storage::Storage;

impl CPU {
    pub fn handle_syscall(&mut self, memory: &Memory, storage: &Storage) -> bool {
        let syscall_id = self.regs[17]; // a7

        // Arguments from a0–a6
        let args = [
            self.regs[10], // a0
            self.regs[11], // a1
            self.regs[12], // a2
            self.regs[13], // a3
            self.regs[14], // a4
            self.regs[15], // a5
            self.regs[16], // a6
        ];

        let result = match syscall_id {
            1 => self.sys_storage_get(args, memory, storage),
            2 => self.sys_storage_set(args, memory, storage),
            3 => self.handle_panic_with_message(memory),
            _ => {
                panic!("Unknown syscall: {}", syscall_id);
            }
        };

        // Return result in a2
        self.regs[12] = result;

        true // continue execution
    }

    pub fn sys_storage_get(&mut self, args: [u32; 7], memory: &Memory, storage: &Storage) -> u32 {
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
                "📦 Storage GET value (len = {}) @ 0x{:08x}: {:02x?}",
                value.len(),
                addr,
                value
            );

            return addr;
        } else {
            println!("❌ Key not found in storage");
            0
        }
    }

    pub fn sys_storage_set(&mut self, args: [u32; 7], memory: &Memory, storage: &Storage) -> u32 {
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

        println!("🔑 Storage SET key: \"{}\" @ 0x{:08x} (len = {})", key, key_ptr, key_len);

        let value_slice_ref = match memory.mem_slice(val_ptr, val_ptr + val_len) {
            Some(r) => r,
            None => return 0,
        };
        let value_slice = value_slice_ref.as_ref();
        storage.set(key, value_slice.to_vec());
        0
    }

    fn handle_panic_with_message(&mut self, memory: &Memory) -> u32 {
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
}
