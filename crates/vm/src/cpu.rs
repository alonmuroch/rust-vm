use std::convert::TryInto;

use crate::decoder::{decode_compressed, decode_full};
use crate::instruction::Instruction;

pub struct CPU {
    pub pc: u32,
    pub regs: [u32; 32],
    pub memory: Vec<u8>,

    // enable verbose logging
    pub verbose: bool,
}

impl CPU {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            memory: code,
            verbose: false,
        }
    }

    pub fn step(&mut self) -> bool {
        match self.next_instruction() {
            Some((instr, size)) => {
                if self.verbose {
                    println!("PC = 0x{:08x}, Instr = {}", self.pc, instr.pretty_print());
                }

                let result = self.execute(instr);
                self.pc = self.pc.wrapping_add(size as u32);
                result
            }
            None => false,
        }
    }

    pub fn next_instruction(&mut self) -> Option<(Instruction, u8)> {
        let pc = self.pc as usize;
        let bytes = &self.memory[pc..];

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

    pub fn execute(&mut self, instr: Instruction) -> bool {
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
                self.regs[rd] = u32::from_le_bytes(self.memory[addr..addr + 4].try_into().unwrap());
            }
            Instruction::Sw { rs1, rs2, offset } => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                self.memory[addr..addr + 4].copy_from_slice(&self.regs[rs2].to_le_bytes());
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
            Instruction::Jal { rd, offset } => {
                self.regs[rd] = self.pc + 4;
                self.pc = self.pc.wrapping_add(offset as u32);
                return true;
            }
            Instruction::Jalr { rd, rs1, offset } => {
                self.regs[rd] = self.pc + 4;
                self.pc = self.regs[rs1].wrapping_add(offset as u32) & !1;
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
            Instruction::Ecall => return false,
            Instruction::Ebreak => return false,
            Instruction::Jr { rs1 } => {
                self.pc = self.regs[rs1];
                return true;
            }
            Instruction::Ret => {
                let target = self.regs[1]; // x1 = ra
                if target == 0 {
                    return false; // halt if ret target is 0
                }
                self.pc = target;
                return true;
            }
            Instruction::Mv { rd, rs2 } => self.regs[rd] = self.regs[rs2],
            Instruction::Addi16sp { imm } => self.regs[2] = self.regs[2].wrapping_add(imm as u32), // x2 is sp
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
            _ => todo!("unhandled instruction"),
        }
        true
    }
}
