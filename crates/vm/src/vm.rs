use crate::cpu::CPU;
use crate::registers::Register;
use crate::memory::{VmMemory};

pub struct VM {
    pub cpu: CPU,
    pub memory: VmMemory,
}

pub const CODE_SIZE_LIMIT: usize = 0x800;

impl VM {
    pub fn new(memory_size: usize) -> Self {
        let memory = VmMemory::new(memory_size);

        let mut regs = [0u32; 32];
        regs[Register::Sp as usize] = memory.stack_top();

        let cpu = CPU {
            pc: 0,
            regs,
            verbose: false,
        };

        Self { cpu, memory }
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
        while self.cpu.step(&self.memory) {}
    }
} 