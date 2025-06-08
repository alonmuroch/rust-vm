pub struct CPU {
    pub pc: u32,
    pub regs: [u32; 32],
    pub memory: Vec<u8>,
}

impl CPU {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            memory: code,
        }
    }

    pub fn step(&mut self) -> bool {
        // decode & execute
        false // for now
    }
}
