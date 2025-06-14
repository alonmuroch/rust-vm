use crate::decoder::{decode_full, decode_compressed};
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::storage::Storage;

pub struct CPU {
    pub pc: u32,
    pub regs: [u32; 32],

    // log
    pub verbose: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            verbose: false,
        }
    }

    pub fn step(&mut self, memory: &Memory, storage: &Storage) -> bool {
        match self.next_instruction(memory) {
            Some((instr, size)) => {
                if self.verbose {
                    println!("PC = 0x{:08x}, Instr = {}", self.pc, instr.pretty_print());
                }
                let old_pc = self.pc;
                let result = self.execute(instr, memory, storage);
                // bump the PC only if the instruction did not change it
                if self.pc == old_pc {
                    self.pc = self.pc.wrapping_add(size as u32);
                }
                result
            }
            None => {
                if let Some(slice_ref) = memory.mem_slice(self.pc as usize, self.pc as usize + 4) {
                    let hex_dump = slice_ref.iter()
                        .map(|b| format!("{:02x}", b)) // still needs deref
                        .collect::<Vec<_>>()
                        .join(" ");

                    eprintln!(
                        "🚨 Unknown or invalid instruction at PC = 0x{:08x} (bytes: [{}])",
                        self.pc,
                        hex_dump
                    );
                } else {
                    eprintln!(
                        "🚨 Unknown or invalid instruction at PC = 0x{:08x} (could not read memory)",
                        self.pc
                    );
                }
                false
            }
        }
    }

    pub fn next_instruction(&mut self, memory: &Memory) -> Option<(Instruction, u8)> {
        if self.pc == 1286 {
            eprintln!("🚨 CPU halted at PC = 0x{:08x}", self.pc);
            return None;
        }
        
        let pc = self.pc as usize;
        let bytes = memory.mem_slice(pc, pc + 4)?;

        if bytes.len() < 2 {
            return None;
        }

        let hword = u16::from_le_bytes([bytes[0], bytes[1]]);
        let is_compressed = (hword & 0b11) != 0b11;

        if is_compressed {
            decode_compressed(hword).map(|inst| (inst, 2))
        } else if bytes.len() >= 4 {
            let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            decode_full(word).map(|inst| (inst, 4))
        } else {
            None
        }
    }

    pub fn execute(&mut self, instr: Instruction, memory: &Memory, storage: &Storage) -> bool {
        match instr {
            Instruction::Add { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2])
            }
            Instruction::Sub { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2])
            }
            Instruction::Addi { rd, rs1, imm } => {
                self.regs[rd] = self.regs[rs1].wrapping_add(imm as u32)
            }
            Instruction::And { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] & self.regs[rs2],
            Instruction::Or { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] | self.regs[rs2],
            Instruction::Xor { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] ^ self.regs[rs2],
            Instruction::Andi { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] & (imm as u32),
            Instruction::Ori { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] | (imm as u32),
            Instruction::Xori { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] ^ (imm as u32),
            Instruction::Slt { rd, rs1, rs2 } => {
                self.regs[rd] = (self.regs[rs1] as i32).lt(&(self.regs[rs2] as i32)) as u32
            }
            Instruction::Sltu { rd, rs1, rs2 } => {
                self.regs[rd] = (self.regs[rs1].lt(&self.regs[rs2])) as u32
            }
            Instruction::Slti { rd, rs1, imm } => {
                self.regs[rd] = (self.regs[rs1] as i32).lt(&imm) as u32
            }
            Instruction::Sltiu { rd, rs1, imm } => {
                let lhs = self.regs[rs1];
                let rhs = imm as u32;
                self.regs[rd] = if lhs < rhs { 1 } else { 0 };
            }
            Instruction::Sll { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1] << (self.regs[rs2] & 0x1F)
            }
            Instruction::Srl { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1] >> (self.regs[rs2] & 0x1F)
            }
            Instruction::Sra { rd, rs1, rs2 } => {
                self.regs[rd] = ((self.regs[rs1] as i32) >> (self.regs[rs2] & 0x1F)) as u32
            }
            Instruction::Slli { rd, rs1, shamt } => self.regs[rd] = self.regs[rs1] << shamt,
            Instruction::Srli { rd, rs1, shamt } => self.regs[rd] = self.regs[rs1] >> shamt,
            Instruction::Srai { rd, rs1, shamt } => {
                self.regs[rd] = ((self.regs[rs1] as i32) >> shamt) as u32
            }
            Instruction::Lw { rd, rs1, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                self.regs[rd] = memory.load_u32(addr);
            }
            Instruction::Lbu { rd, rs1, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                let byte = memory.load_byte(addr);
                self.regs[rd] = byte as u32;
            }
            Instruction::Lh { rd, rs1, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                let halfword = memory.load_halfword(addr); // returns u16
                let value = (halfword as i16) as i32 as u32; // sign-extend to 32-bit
                self.regs[rd] = value;
            }

            Instruction::Sh { rs1, rs2, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.store_u16(addr, (self.regs[rs2] & 0xFFFF) as u16);
            }
            Instruction::Sw { rs1, rs2, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.store_u32(addr, self.regs[rs2]);
            }
            Instruction::Sb { rs1, rs2, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.store_u8(addr, (self.regs[rs2] & 0xFF) as u8);
            }
        
            Instruction::Beq { rs1, rs2, offset } => {
                if self.regs[rs1] == self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bne { rs1, rs2, offset } => {
                if self.regs[rs1] != self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Blt { rs1, rs2, offset } => {
                if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bge { rs1, rs2, offset } => {
                if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bltu { rs1, rs2, offset } => {
                if self.regs[rs1] < self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }

            Instruction::Bgeu { rs1, rs2, offset } => {
                if self.regs[rs1] >= self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Jal { rd, offset } => {
                self.regs[rd] = self.pc + 4;
                self.pc = self.pc.wrapping_add(offset as u32);
                return true;
            }
            Instruction::Jalr { rd, rs1, offset } => {
                let base = self.regs[rs1];
                let target = base.wrapping_add(offset as u32) & !1;
                let return_address = self.pc + 4;

                if rd != 0 {
                    self.regs[rd] = return_address;
                } 

                self.pc = target;
                return true;
            }

            Instruction::Lui { rd, imm } => self.regs[rd] = (imm << 12) as u32,
            Instruction::Auipc { rd, imm } => {
                self.regs[rd] = self.pc.wrapping_add((imm << 12) as u32)
            }
            Instruction::Mul { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2])
            }
            Instruction::Mulh { rd, rs1, rs2 } => {
                self.regs[rd] = (((self.regs[rs1] as i64) * (self.regs[rs2] as i64)) >> 32) as u32
            }
            Instruction::Mulhu { rd, rs1, rs2 } => {
                self.regs[rd] = (((self.regs[rs1] as u64) * (self.regs[rs2] as u64)) >> 32) as u32
            }
            Instruction::Mulhsu { rd, rs1, rs2 } => {
                self.regs[rd] =
                    (((self.regs[rs1] as i64) * (self.regs[rs2] as u64 as i64)) >> 32) as u32
            }
            Instruction::Div { rd, rs1, rs2 } => {
                self.regs[rd] = (self.regs[rs1] as i32).wrapping_div(self.regs[rs2] as i32) as u32
            }
            Instruction::Divu { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] / self.regs[rs2],
            Instruction::Rem { rd, rs1, rs2 } => {
                self.regs[rd] = (self.regs[rs1] as i32).wrapping_rem(self.regs[rs2] as i32) as u32
            }
            Instruction::Remu { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] % self.regs[rs2],
            Instruction::Ecall => {
                return self.handle_syscall(memory, storage);
            }
            Instruction::Ebreak => {
                return false
            }
            Instruction::Jr { rs1 } => {
                self.pc = self.regs[rs1];
                return true;
            }
            Instruction::Ret => {
                let target = self.regs[1]; // x1 = ra
                if target == 0 || target == 0xFFFF_FFFF {
                    return false; // halt if ret target is 0
                }
    
                self.pc = target;
                return true;
            }
            Instruction::Mv { rd, rs2 } => self.regs[rd] = self.regs[rs2],
            Instruction::Addi16sp { imm } => self.regs[2] = self.regs[2].wrapping_add(imm as u32), // x2 is sp
            Instruction::Addi4spn { rd, imm } => {
                self.regs[rd] = self.regs[2].wrapping_add(imm);
            }
            Instruction::Nop => {}
            Instruction::Beqz { rs1, offset } => {
                if self.regs[rs1] == 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bnez { rs1, offset } => {
                if self.regs[rs1] != 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }

            Instruction::MiscAlu { rd, rs2, op } => {
                match op {
                    crate::instruction::MiscAluOp::Sub => {
                        self.regs[rd] = self.regs[rd].wrapping_sub(self.regs[rs2]);
                    }
                    crate::instruction::MiscAluOp::Xor => {
                        self.regs[rd] = self.regs[rd] ^ self.regs[rs2];
                    }
                    crate::instruction::MiscAluOp::Or => {
                        self.regs[rd] = self.regs[rd] | self.regs[rs2];
                    }
                    crate::instruction::MiscAluOp::And => {
                        self.regs[rd] = self.regs[rd] & self.regs[rs2];
                    }
                }
            }
            _ => todo!("unhandled instruction"),
        }
        true

    }               
}
