use crate::cpu::CPU;

impl CPU {
    pub fn run_with_trace(&mut self, max_cycles: usize) -> String {
        println!("--- Begin execution trace ---");

        for _ in 0..max_cycles {
            if !self.step() {
                println!("Halted");
                break;
            }
        }

        println!("--- End execution trace ---\n");
        self.dump_registers()
    }

    pub fn dump_registers(&self) -> String {
        let mut out = String::from("Registers:\n");
        for i in 0..32 {
            out.push_str(&format!("x{:02} = 0x{:08x}\n", i, self.regs[i]));
        }
        out
    }
}
