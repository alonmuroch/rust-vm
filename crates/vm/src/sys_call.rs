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
            _ => {
                panic!("Unknown syscall: {}", syscall_id);
            }
        };

        // Return result in a0
        self.regs[10] = result;

        true // continue execution
    }

    pub fn sys_storage_get(&mut self, args: [u32; 7], memory: &Memory, storage: &Storage) -> u32 {
        let key_ptr = args[0] as usize;
        let key_len = args[1] as usize;

        // Safely get the key slice from memory
        let key_slice_ref = match memory.mem_slice(key_ptr, key_ptr + key_len) {
            Some(r) => r,
            None => return 0,
        };
        let key_slice = key_slice_ref.as_ref();

        // Convert the key slice to a &str
        let key = match core::str::from_utf8(key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        // Lookup the value in storage
        if let Some(value) = storage.get(key) {
            // Prepare buffer: [len (u32 LE)] + [value bytes]
            let mut buf = (value.len() as u32).to_le_bytes().to_vec();
            buf.extend_from_slice(value.as_slice());

            // Allocate in guest memory and copy
            let addr = memory.alloc_on_heap(&buf);
            addr
        } else {
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
        let key = match core::str::from_utf8(key_slice) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let value_slice_ref = match memory.mem_slice(val_ptr, val_ptr + val_len) {
            Some(r) => r,
            None => return 0,
        };
        let value_slice = value_slice_ref.as_ref();
        storage.set(key, value_slice.to_vec());
        0
    }
}
