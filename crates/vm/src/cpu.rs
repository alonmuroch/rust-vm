use crate::instruction::Instruction;
use crate::decoder::decode_full;
use std::convert::TryInto;

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
        if self.pc as usize + 4 > self.memory.len() {
            return false;
        }

        let word = u32::from_le_bytes([
            self.memory[self.pc as usize],
            self.memory[self.pc as usize + 1],
            self.memory[self.pc as usize + 2],
            self.memory[self.pc as usize + 3],
        ]);

        match decode_full(word) {
            Some(Instruction::Add { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]),
            Some(Instruction::Sub { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]),
            Some(Instruction::Addi { rd, rs1, imm }) => self.regs[rd] = self.regs[rs1].wrapping_add(imm as u32),
            Some(Instruction::And { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] & self.regs[rs2],
            Some(Instruction::Or { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] | self.regs[rs2],
            Some(Instruction::Xor { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] ^ self.regs[rs2],
            Some(Instruction::Andi { rd, rs1, imm }) => self.regs[rd] = self.regs[rs1] & (imm as u32),
            Some(Instruction::Ori { rd, rs1, imm }) => self.regs[rd] = self.regs[rs1] | (imm as u32),
            Some(Instruction::Xori { rd, rs1, imm }) => self.regs[rd] = self.regs[rs1] ^ (imm as u32),
            Some(Instruction::Slt { rd, rs1, rs2 }) => self.regs[rd] = (self.regs[rs1] as i32).lt(&(self.regs[rs2] as i32)) as u32,
            Some(Instruction::Slti { rd, rs1, imm }) => self.regs[rd] = (self.regs[rs1] as i32).lt(&imm) as u32,
            Some(Instruction::Sll { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] << (self.regs[rs2] & 0x1F),
            Some(Instruction::Srl { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] >> (self.regs[rs2] & 0x1F),
            Some(Instruction::Sra { rd, rs1, rs2 }) => self.regs[rd] = ((self.regs[rs1] as i32) >> (self.regs[rs2] & 0x1F)) as u32,
            Some(Instruction::Slli { rd, rs1, shamt }) => self.regs[rd] = self.regs[rs1] << shamt,
            Some(Instruction::Srli { rd, rs1, shamt }) => self.regs[rd] = self.regs[rs1] >> shamt,
            Some(Instruction::Srai { rd, rs1, shamt }) => self.regs[rd] = ((self.regs[rs1] as i32) >> shamt) as u32,
            Some(Instruction::Lw { rd, rs1, offset }) => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                self.regs[rd] = u32::from_le_bytes(self.memory[addr..addr + 4].try_into().unwrap());
            }
            Some(Instruction::Sw { rs1, rs2, offset }) => {
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                self.memory[addr..addr + 4].copy_from_slice(&self.regs[rs2].to_le_bytes());
            }
            Some(Instruction::Beq { rs1, rs2, offset }) => {
                if self.regs[rs1] == self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(Instruction::Bne { rs1, rs2, offset }) => {
                if self.regs[rs1] != self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(Instruction::Blt { rs1, rs2, offset }) => {
                if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(Instruction::Bge { rs1, rs2, offset }) => {
                if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(Instruction::Jal { rd, offset }) => {
                self.regs[rd] = self.pc + 4;
                self.pc = self.pc.wrapping_add(offset as u32);
                return true;
            }
            Some(Instruction::Jalr { rd, rs1, offset }) => {
                self.regs[rd] = self.pc + 4;
                self.pc = self.regs[rs1].wrapping_add(offset as u32) & !1;
                return true;
            }
            Some(Instruction::Lui { rd, imm }) => self.regs[rd] = (imm << 12) as u32,
            Some(Instruction::Auipc { rd, imm }) => self.regs[rd] = self.pc.wrapping_add((imm << 12) as u32),
            Some(Instruction::Mul { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]),
            Some(Instruction::Mulh { rd, rs1, rs2 }) => self.regs[rd] = (((self.regs[rs1] as i64) * (self.regs[rs2] as i64)) >> 32) as u32,
            Some(Instruction::Mulhu { rd, rs1, rs2 }) => self.regs[rd] = (((self.regs[rs1] as u64) * (self.regs[rs2] as u64)) >> 32) as u32,
            Some(Instruction::Mulhsu { rd, rs1, rs2 }) => self.regs[rd] = (((self.regs[rs1] as i64) * (self.regs[rs2] as u64 as i64)) >> 32) as u32,
            Some(Instruction::Div { rd, rs1, rs2 }) => self.regs[rd] = (self.regs[rs1] as i32).wrapping_div(self.regs[rs2] as i32) as u32,
            Some(Instruction::Divu { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] / self.regs[rs2],
            Some(Instruction::Rem { rd, rs1, rs2 }) => self.regs[rd] = (self.regs[rs1] as i32).wrapping_rem(self.regs[rs2] as i32) as u32,
            Some(Instruction::Remu { rd, rs1, rs2 }) => self.regs[rd] = self.regs[rs1] % self.regs[rs2],
            Some(Instruction::Ecall) => return false,
            Some(Instruction::Ebreak) => return false,
            Some(Instruction::Jr { rs1 }) => {
                self.pc = self.regs[rs1];
                return true;
            }
            Some(Instruction::Ret) => {
                self.pc = self.regs[1];
                return true;
            }
            Some(Instruction::Mv { rd, rs2 }) => self.regs[rd] = self.regs[rs2],
            Some(Instruction::Addi16sp { imm }) => self.regs[2] = self.regs[2].wrapping_add(imm as u32),
            Some(Instruction::Nop) => {},
            Some(Instruction::Beqz { rs1, offset }) => {
                if self.regs[rs1] == 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(Instruction::Bnez { rs1, offset }) => {
                if self.regs[rs1] != 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Some(_) => todo!("unhandled instruction"),
            None => return false,
        }

        self.pc += 4;
        true
    }
}
