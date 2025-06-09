use crate::instruction::Instruction;
use crate::isa::Opcode;
use crate::isa_compressed::CompressedOpcode;

/// Unified decoder for either 16-bit compressed or 32-bit instruction.
///
/// - Expects 4 bytes (`[u8; 4]`)
/// - Returns decoded `Instruction` and how many bytes were consumed (2 or 4)
pub fn decode(bytes: &[u8]) -> Option<(Instruction, u8)> {
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

pub fn decode_full(word: u32) -> Option<Instruction> {
    let opcode = Opcode::from_u8((word & 0x7f) as u8)?;

    let rd = ((word >> 7) & 0x1f) as usize;
    let funct3 = ((word >> 12) & 0x07) as u8;
    let rs1 = ((word >> 15) & 0x1f) as usize;
    let rs2 = ((word >> 20) & 0x1f) as usize;
    let funct7 = ((word >> 25) & 0x7f) as u8;

    match opcode {
        Opcode::Op => match (funct3, funct7) {
            (0x0, 0x00) => Some(Instruction::Add { rd, rs1, rs2 }),
            (0x0, 0x20) => Some(Instruction::Sub { rd, rs1, rs2 }),
            (0x1, 0x00) => Some(Instruction::Sll { rd, rs1, rs2 }),
            (0x2, 0x00) => Some(Instruction::Slt { rd, rs1, rs2 }),
            (0x3, 0x00) => Some(Instruction::Sltu { rd, rs1, rs2 }),
            (0x4, 0x00) => Some(Instruction::Xor { rd, rs1, rs2 }),
            (0x5, 0x00) => Some(Instruction::Srl { rd, rs1, rs2 }),
            (0x5, 0x20) => Some(Instruction::Sra { rd, rs1, rs2 }),
            (0x6, 0x00) => Some(Instruction::Or { rd, rs1, rs2 }),
            (0x7, 0x00) => Some(Instruction::And { rd, rs1, rs2 }),
            (0x0, 0x01) => Some(Instruction::Mul { rd, rs1, rs2 }),
            (0x1, 0x01) => Some(Instruction::Mulh { rd, rs1, rs2 }),
            (0x2, 0x01) => Some(Instruction::Mulhsu { rd, rs1, rs2 }),
            (0x3, 0x01) => Some(Instruction::Mulhu { rd, rs1, rs2 }),
            (0x4, 0x01) => Some(Instruction::Div { rd, rs1, rs2 }),
            (0x5, 0x01) => Some(Instruction::Divu { rd, rs1, rs2 }),
            (0x6, 0x01) => Some(Instruction::Rem { rd, rs1, rs2 }),
            (0x7, 0x01) => Some(Instruction::Remu { rd, rs1, rs2 }),
            _ => None,
        },
        Opcode::OpImm => {
            let imm = (word as i32) >> 20;
            match funct3 {
                0x0 => Some(Instruction::Addi { rd, rs1, imm }),
                0x2 => Some(Instruction::Slti { rd, rs1, imm }),
                0x4 => Some(Instruction::Xori { rd, rs1, imm }),
                0x6 => Some(Instruction::Ori { rd, rs1, imm }),
                0x7 => Some(Instruction::Andi { rd, rs1, imm }),
                0x1 => Some(Instruction::Slli { 
                    rd,
                    rs1,
                    shamt: (imm & 0x1f) as u8,
                }),
                0x5 => match funct7 {
                    0x00 => Some(Instruction::Srli { 
                        rd,
                        rs1,
                        shamt: (imm & 0x1f) as u8,
                    }),
                    0x20 => Some(Instruction::Srai { 
                        rd,
                        rs1,
                        shamt: (imm & 0x1f) as u8,
                    }),
                    _ => None,
                },
                _ => None,
            }

        },
        Opcode::Load => {
            let imm = (word as i32) >> 20;
            match funct3 {
                0x2 => Some(Instruction::Lw { 
                    rd, 
                    rs1, 
                    offset: imm, 
                }),
                _ => None,
            }
        },
        Opcode::Store => {
            // Extract 12-bit signed immediate for S-type (e.g., sw)
            let imm11_5 = ((word >> 25) & 0x7f) << 5;
            let imm4_0 = (word >> 7) & 0x1f;
            let imm = ((imm11_5 | imm4_0) as i32) << 20 >> 20; // sign-extend 12-bit

            match funct3 {
                0x0 => Some(Instruction::Sb { 
                    rs1, 
                    rs2, 
                    offset: imm,
                }),
                0x2 => Some(Instruction::Sw { 
                    rs1, 
                    rs2, 
                    offset: imm,
                }),
                _ => None,
            }
        },
        Opcode::Branch => {
            let imm = extract_branch_offset(word);
            match funct3 {
                0x0 => Some(Instruction::Beq { 
                    rs1, 
                    rs2, 
                    offset: imm, 
                }),
                0x1 => Some(Instruction::Bne { 
                    rs1, 
                    rs2, 
                    offset: imm, 
                }),
                0x4 => Some(Instruction::Blt { 
                    rs1, 
                    rs2, 
                    offset: imm, 
                }),
                0x5 => Some(Instruction::Bge { 
                    rs1, 
                    rs2, 
                    offset: imm, 
                }),
                0x6 => Some(Instruction::Bltu {
                    rs1,
                    rs2,
                    offset: imm,
                }),
                0x7 => Some(Instruction::Bgeu {
                    rs1,
                    rs2,
                    offset: imm,
                }),
                _ => None,
            }
        },
        Opcode::Jal => {
            let imm = extract_jal_offset(word);
            Some(Instruction::Jal { 
                rd, 
                offset: imm, 
            })
        },
        Opcode::Jalr => {
            let imm = (word as i32) >> 20;
            Some(Instruction::Jalr { 
                rd, 
                rs1, 
                offset: imm, 
            })
        },
        Opcode::Lui => {
            let imm = (word & 0xfffff000) as i32;
            Some(Instruction::Lui { 
                rd,
                imm: imm as i32, 
            })
        },
        Opcode::Auipc => {
            let imm = (word & 0xfffff000) as i32;
            Some(Instruction::Auipc { rd, imm })
        },
        Opcode::System => Some(Instruction::Ecall),
        _ => {
            println!("Unimplemented instruction: {:?}", opcode);
            todo!("implement: {:?}", opcode);
        }
    }
}

/// Decode a 16-bit RISC-V compressed instruction into a full Instruction.
///
/// This function currently supports a minimal set:
/// - C.ADDI, C.LI, C.LW, C.SW, C.JAL, C.JR, C.RET
pub fn decode_compressed(hword: u16) -> Option<Instruction> {
    let funct3 = (hword >> 13) & 0b111;
    let opcode = hword & 0b11;
    let rd = ((hword >> 7) & 0x1f) as usize;
    let rs1 = rd;
    let rs2 = ((hword >> 2) & 0x1f) as usize;

    let op = CompressedOpcode::from_bits(funct3, opcode)?;

    match op {
        CompressedOpcode::Addi => {
            let imm = (((hword >> 2) & 0b11111) | (((hword >> 12) & 0x1) << 5)) as i32;
            let imm = (imm << 26) >> 26; // sign-extend
            Some(Instruction::Addi { rd, rs1, imm })
        }

        CompressedOpcode::Li => {
            let imm = (((hword >> 2) & 0b11111) | (((hword >> 12) & 0x1) << 5)) as i32;
            let imm = (imm << 26) >> 26;
            Some(Instruction::Addi { rd, rs1: 0, imm })
        }

        CompressedOpcode::LuiOrAddi16sp => {
            if rd == 2 {
                // C.ADDI16SP
                let imm = (
                    ((hword >> 12) & 0x1) << 5 |
                    ((hword >> 6) & 0x1) << 4 |
                    ((hword >> 5) & 0x1) << 9 |
                    ((hword >> 3) & 0x3) << 6 |
                    ((hword >> 2) & 0x1) << 8) as i32;
                let imm = (imm << 23) >> 23; // sign-extend
                Some(Instruction::Addi16sp { imm })
            } else if rd != 0 {
                // C.LUI
                let imm = (((hword >> 2) & 0x1F) | ((hword >> 12) & 0x1) << 5) as u32;
                let imm = imm << 12;
                Some(Instruction::Lui { rd, imm: imm as i32 })
            } else {
                None
            }
        }

        CompressedOpcode::Addi4spn => {
            let rd = 8 + ((hword >> 2) & 0b111) as usize;

            if rd == 0 {
                return None; // reserved
            }

            let imm =
                ((hword >>  6) & 0b0001) << 2  |  // bit 2
                ((hword >>  5) & 0b0001) << 3  |  // bit 3
                ((hword >> 11) & 0b0001) << 4  |  // bit 4
                ((hword >>  7) & 0b1111) << 6;    // bits 6–9

            if imm == 0 {
                return None; // nzuimm must be non-zero
            }

            Some(Instruction::Addi4spn { rd, imm: imm as u32 })
        }

        CompressedOpcode::Jal => {
            let imm = decode_cj_imm(hword); // implement CJ immediate decoder
            Some(Instruction::Jal { rd: 1, offset: imm }) // rd = x1
        }

        CompressedOpcode::J => {
            let imm = decode_cj_imm(hword);
            Some(Instruction::Jal { rd: 0, offset: imm }) // jump without link
        }

        CompressedOpcode::RegOrJump => {
            match (rs1, rs2) {
                (0, 0) => Some(Instruction::Ebreak),
                (1, 0) => Some(Instruction::Ret),
                (_, 0) => Some(Instruction::Jr { rs1 }),
                (_, _) => {
                    if rs1 == rs2 {
                        Some(Instruction::Mv { rd: rs1, rs2 })
                    } else {
                        Some(Instruction::Add { rd: rs1, rs1, rs2 })
                    }
                }
            }
        }

        CompressedOpcode::Slli => {
            let shamt = ((hword >> 2) & 0b11111) as i32;
            Some(Instruction::Addi { rd, rs1, imm: shamt }) // emulate as ADDI with left shift beforehand
        }

        CompressedOpcode::Beqz | CompressedOpcode::Bnez => {
            // You may decode it, but usually this is handled in control flow
            None
        }

        CompressedOpcode::Sw => {
            let rd_ = ((hword >> 2) & 0b111) + 8;   // rs2'
            let rs1_ = ((hword >> 7) & 0b111) + 8;  // rs1'
            let imm = ((hword >> 10) & 0b111) << 3 | ((hword >> 5) & 0b11) << 6;
            let offset = (imm as i32); // no sign-extension needed
            Some(Instruction::Sw {
                rs1: rs1_ as usize,
                rs2: rd_ as usize,
                offset,
            })
        }

        CompressedOpcode::Lw => {
            let rd_ = ((hword >> 2) & 0b111) + 8;    // rd'
            let rs1_ = ((hword >> 7) & 0b111) + 8;   // rs1'

            // imm[5|4:3|2] → bits: [5] bit 5 = bit 5, [4:3] bits 11:10, [2] bit 6
            let imm = ((hword >> 6) & 0b1) << 2      // imm[2]
                    | ((hword >> 10) & 0b11) << 3    // imm[4:3]
                    | ((hword >> 5) & 0b1) << 6;     // imm[5]

            let offset = imm as i32;

            Some(Instruction::Lw {
                rd: rd_ as usize,
                rs1: rs1_ as usize,
                offset,
            })
        }


        CompressedOpcode::Swsp => {
            let rs2 = ((hword >> 2) & 0x1F) as usize;
            let imm = (((hword >> 7) & 0x1F) << 2) | (((hword >> 9) & 0x3) << 6);
            Some(Instruction::Sw { rs1: 2, rs2, offset: imm as i32 }) // rs1 = sp (x2)
        }

        _ => {
            println!("Unimplemented compressed instruction: {:?}", op);
            todo!("implement: {:?}", op);
        }
    }
}

fn decode_cj_imm(hword: u16) -> i32 {
    let imm = (
        ((hword >> 12) & 0b1) << 11 |
        ((hword >> 11) & 0b1) << 4  |
        ((hword >> 9)  & 0b11) << 8 |
        ((hword >> 8)  & 0b1) << 10 |
        ((hword >> 7)  & 0b1) << 6  |
        ((hword >> 6)  & 0b1) << 7  |
        ((hword >> 3)  & 0b111) << 1 |
        ((hword >> 2)  & 0b1) << 5
    ) as i32;
    (imm << 20) >> 20 // sign-extend 12-bit
}

fn extract_branch_offset(word: u32) -> i32 {
    let imm12 = ((word >> 31) & 0x1) << 12;
    let imm10_5 = ((word >> 25) & 0x3f) << 5;
    let imm4_1 = ((word >> 8) & 0xf) << 1;
    let imm11 = ((word >> 7) & 0x1) << 11;
    let imm = (imm12 | imm11 | imm10_5 | imm4_1) as i32;
    (imm << 19) >> 19
}

fn extract_jal_offset(word: u32) -> i32 {
    let imm20 = ((word >> 31) & 0x1) << 20;
    let imm10_1 = ((word >> 21) & 0x3ff) << 1;
    let imm11 = ((word >> 20) & 0x1) << 11;
    let imm19_12 = ((word >> 12) & 0xff) << 12;
    let imm = (imm20 | imm19_12 | imm11 | imm10_1) as i32;
    (imm << 11) >> 11
}
