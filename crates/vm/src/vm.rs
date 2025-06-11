use crate::cpu::CPU;
use crate::registers::Register;
use crate::memory::{Memory};
use crate::global::Config;
use crate::storage::{self, Storage};

pub struct VM {
    pub cpu: CPU,
    pub memory: Memory,
    pub storage: Storage,
}

pub const CODE_SIZE_LIMIT: usize = 0x800;

impl VM {
    pub fn new(memory_size: usize) -> Self {
        let memory = Memory::new(memory_size);

        let mut regs = [0u32; 32];
        regs[Register::Sp as usize] = memory.stack_top();

        let cpu = CPU {
            pc: 0,
            regs,
            verbose: false,
        };

        let storage = Storage::new();

        Self { cpu, memory, storage }
    }

    pub fn set_code(&mut self, code: &[u8]) {
        self.memory.write_code(code);
        self.cpu.pc = 0;
    }

    pub fn alloc_and_write(&mut self, data: &[u8]) -> u32 {
        self.memory.alloc_on_heap(data)
    }

    pub fn set_reg_to_data(&mut self, reg: Register, data: &[u8]) -> u32 {
        let addr = self.alloc_and_write(data);
        self.cpu.regs[reg as usize] = addr;
        addr
    }

    pub fn set_reg_u32(&mut self, reg: Register, data: u32) {
        self.cpu.regs[reg as usize] = data;
    }

    pub fn dump_memory(&self, start: usize, end: usize) {
        assert!(start < end, "invalid memory range");
        assert!(end <= self.memory.mem().len(), "range out of bounds");

        for addr in (start..end).step_by(16) {
            let line = &self.memory.mem()[addr..end.min(addr + 16)];

            let hex: Vec<String> = line.iter().map(|b| format!("{:02x}", b)).collect();
            let hex_str = hex.join(" ");

            let ascii: String = line.iter()
                .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                .collect();

            println!("{:08x}  {:<47}  |{}|", addr, hex_str, ascii);
        }
    }

    pub fn run(&mut self) {
        // Validate pubkey pointer (A0)
        let pubkey_ptr = self.cpu.regs[Register::A0 as usize] as usize;
        if pubkey_ptr == 0 {
            panic!("Entrypoint: pubkey pointer is not set");
        }

        // Validate input length (A2)
        let input_len = self.cpu.regs[Register::A2 as usize] as usize;
        if input_len > Config::MAX_INPUT_LEN {
            panic!(
                "Entrypoint: input length {} exceeds MAX_INPUT_LEN ({})",
                input_len,
                Config::MAX_INPUT_LEN
            );
        }

        // Validate result pointer (A3)
        let result_ptr = self.cpu.regs[Register::A3 as usize] as usize;
        if result_ptr == 0 {
            panic!("Entrypoint: result pointer is not set");
        }
        if result_ptr >= self.memory.mem().len() {
            panic!("Entrypoint: result pointer is out of memory bounds");
        }

        while self.cpu.step(&self.memory, &self.storage) {}
    }
} 