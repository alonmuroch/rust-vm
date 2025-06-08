use crate::cpu::CPU;
use crate::decoder::decode;

impl CPU {
    pub fn run_with_trace(&mut self, max_cycles: usize) -> String {
        println!("--- Begin execution trace ---");

        for cycle in 0..max_cycles {
            let pc = self.pc as usize;
            let mem = &self.memory[pc..];

            println!(
                "mem[0] = 0x{:02x}, mem[1] = 0x{:02x}, hword = 0x{:04x}, hword_reverse = 0x{:04x}",
                mem[0],
                mem[1],
                u16::from_le_bytes([mem[0], mem[1]]),
                u16::from_le_bytes([mem[1], mem[0]])
            );

            match decode(mem) {
                Some((instruction, size)) => {
                    let encoded = &mem[..size as usize];
                    let hex = match size {
                        2 => u16::from_le_bytes([encoded[0], encoded[1]]) as u32,
                        4 => u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]),
                        _ => 0,
                    };
                    println!(
                        "[Cycle {:03}] PC = 0x{:08x}, Instr = 0x{:0width$x} ({} bytes) => {:?}",
                        cycle,
                        self.pc,
                        hex,
                        size,
                        instruction,
                        width = (size * 2) as usize,
                    );
                }
                None => {
                    println!("[Cycle {:03}] PC = 0x{:08x}, Invalid instruction", cycle, self.pc);
                    break;
                }
            }

            if !self.step() {
                println!("[Cycle {:03}] Halted", cycle);
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
