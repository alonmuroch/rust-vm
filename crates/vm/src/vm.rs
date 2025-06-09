use crate::cpu::CPU;
use crate::registers::Register;

pub struct VM {
    pub cpu: CPU,
    pub next_free: u32, // Tracks next free byte in memory
}

 pub const CODE_SIZE_LIMIT: usize = 0x800; // Max code section: 2 KB
 pub const STACK_SIZE: usize = 0x200; // Max code section: 512 bytes
 pub const STACK_OFFSET_FROM_TOP: usize = STACK_SIZE / 2;

impl VM {
    pub fn new(memory_size: usize) -> Self {
        assert!(
            memory_size >= CODE_SIZE_LIMIT,
            "memory_size must be at least CODE_LIMIT ({} bytes)",
            CODE_SIZE_LIMIT
        );

        let mut regs = [0u32; 32];
        // Set SP to halfway down from the top of memory
        regs[Register::Sp as usize] = (memory_size - STACK_OFFSET_FROM_TOP) as u32;

        Self {
            cpu: CPU {
                pc: 0,
                regs,
                memory: vec![0; memory_size],
                verbose: false,
            },
            next_free: CODE_SIZE_LIMIT as u32, // Start allocating data after code
        }
    }


    /// Load the RISC-V code into memory at address 0
    pub fn set_code(&mut self, code: &[u8]) {
        assert!(
            code.len() <= CODE_SIZE_LIMIT,
            "code size ({}) exceeds CODE_LIMIT ({:#x})",
            code.len(),
            CODE_SIZE_LIMIT
        );
        self.cpu.memory[0..code.len()].copy_from_slice(code);
        self.cpu.pc = 0; // entrypoint
    }

    /// Allocate space and write arbitrary data into memory
    pub fn alloc_and_write(&mut self, data: &[u8]) -> u32 {
        let addr = self.next_free;
        let end = addr + data.len() as u32;
        assert!(
            end as usize <= self.cpu.memory.len(),
            "memory overflow: tried to write {} bytes at {:#x}",
            data.len(),
            addr
        );
        self.cpu.memory[addr as usize..end as usize].copy_from_slice(data);
        self.next_free = end;
        addr
    }

    /// Set a register to point to allocated memory containing `data`
    pub fn set_reg_to_data(&mut self, reg: Register, data: &[u8]) -> u32 {
        let addr = self.alloc_and_write(data);
        self.cpu.regs[reg as usize] = addr;
        addr
    }

    /// Dump a range of memory as hex + ASCII
    pub fn dump_memory(&self, start: usize, end: usize) {
        assert!(start < end, "invalid memory range");
        assert!(end <= self.cpu.memory.len(), "range out of bounds");

        for addr in (start..end).step_by(16) {
            let line = &self.cpu.memory[addr..end.min(addr + 16)];

            let hex: Vec<String> = line.iter().map(|b| format!("{:02x}", b)).collect();
            let hex_str = hex.join(" ");

            let ascii: String = line.iter()
                .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                .collect();

            println!("{:08x}  {:<47}  |{}|", addr, hex_str, ascii);
        }
    }

    pub fn run(&mut self) {
        while self.cpu.step() {}
    }
}
