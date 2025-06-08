pub mod cpu;
pub mod memory;
pub mod decode;

use cpu::CPU;

pub struct VM {
    cpu: CPU,
}

impl VM {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            cpu: CPU::new(code),
        }
    }

    pub fn run(&mut self) {
        while self.cpu.step() {}
    }
}
