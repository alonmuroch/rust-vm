/// Module for extracting and formatting RISC-V instructions from ELF binaries.
/// 
/// This module uses the existing infrastructure from the compiler and VM crates:
/// - `compiler::parse_elf_from_bytes` for ELF parsing
/// - `vm::decoder::decode` for RISC-V instruction decoding
/// - `vm::instruction::Instruction` for instruction representation
/// 
/// The purpose is to extract instructions from ELF files in the same way
/// the VM would decode them, allowing for accurate comparison.

use anyhow::{Context, Result};
use compiler::parse_elf_from_bytes;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use vm::decoder;
use vm::instruction::Instruction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElfInstruction {
    pub pc: u32,
    pub raw: u32,
    pub decoded: String,
}

/// Decode ELF binary and extract RISC-V instructions using existing decoders
pub fn decode_elf(path: &Path) -> Result<Vec<ElfInstruction>> {
    let buffer = fs::read(path)
        .with_context(|| format!("Failed to read ELF file: {:?}", path))?;
    
    // Use the compiler's ELF parser
    let elf_info = parse_elf_from_bytes(&buffer)
        .with_context(|| format!("Failed to parse ELF file: {:?}", path))?;
    
    let mut instructions = Vec::new();
    
    // Get the flat code section
    if let Some((code, base_addr)) = elf_info.get_flat_code() {
        let mut offset = 0;
        
        while offset < code.len() {
            let pc = base_addr as u32 + offset as u32;
            let remaining = &code[offset..];
            
            // Use VM's decoder to decode the instruction
            if let Some((instruction, size)) = decoder::decode(remaining) {
                let raw = match size {
                    2 => {
                        // 16-bit compressed instruction
                        u16::from_le_bytes([remaining[0], remaining[1]]) as u32
                    }
                    4 => {
                        // 32-bit instruction
                        u32::from_le_bytes([
                            remaining[0],
                            remaining[1],
                            remaining[2],
                            remaining[3],
                        ])
                    }
                    _ => 0,
                };
                
                let decoded = format_instruction(&instruction);
                
                instructions.push(ElfInstruction {
                    pc,
                    raw,
                    decoded,
                });
                
                offset += size as usize;
            } else {
                // If we can't decode, skip 2 bytes (minimum instruction size)
                offset += 2;
            }
        }
    }
    
    // Also decode rodata if needed (for data analysis)
    // This could be useful for comparing constant data access
    
    Ok(instructions)
}

/// Format an instruction for display
fn format_instruction(inst: &Instruction) -> String {
    use vm::instruction::Instruction::*;
    
    match inst {
        // R-type instructions
        Add { rd, rs1, rs2 } => format!("add x{}, x{}, x{}", rd, rs1, rs2),
        Sub { rd, rs1, rs2 } => format!("sub x{}, x{}, x{}", rd, rs1, rs2),
        Xor { rd, rs1, rs2 } => format!("xor x{}, x{}, x{}", rd, rs1, rs2),
        Or { rd, rs1, rs2 } => format!("or x{}, x{}, x{}", rd, rs1, rs2),
        And { rd, rs1, rs2 } => format!("and x{}, x{}, x{}", rd, rs1, rs2),
        Sll { rd, rs1, rs2 } => format!("sll x{}, x{}, x{}", rd, rs1, rs2),
        Srl { rd, rs1, rs2 } => format!("srl x{}, x{}, x{}", rd, rs1, rs2),
        Sra { rd, rs1, rs2 } => format!("sra x{}, x{}, x{}", rd, rs1, rs2),
        Slt { rd, rs1, rs2 } => format!("slt x{}, x{}, x{}", rd, rs1, rs2),
        Sltu { rd, rs1, rs2 } => format!("sltu x{}, x{}, x{}", rd, rs1, rs2),
        
        // I-type instructions
        Addi { rd, rs1, imm } => format!("addi x{}, x{}, {}", rd, rs1, imm),
        Xori { rd, rs1, imm } => format!("xori x{}, x{}, {}", rd, rs1, imm),
        Ori { rd, rs1, imm } => format!("ori x{}, x{}, {}", rd, rs1, imm),
        Andi { rd, rs1, imm } => format!("andi x{}, x{}, {}", rd, rs1, imm),
        Slli { rd, rs1, shamt } => format!("slli x{}, x{}, {}", rd, rs1, shamt),
        Srli { rd, rs1, shamt } => format!("srli x{}, x{}, {}", rd, rs1, shamt),
        Srai { rd, rs1, shamt } => format!("srai x{}, x{}, {}", rd, rs1, shamt),
        Slti { rd, rs1, imm } => format!("slti x{}, x{}, {}", rd, rs1, imm),
        Sltiu { rd, rs1, imm } => format!("sltiu x{}, x{}, {}", rd, rs1, imm),
        
        // Load instructions  
        Lb { rd, rs1, offset } => format!("lb x{}, {}(x{})", rd, offset, rs1),
        Lh { rd, rs1, offset } => format!("lh x{}, {}(x{})", rd, offset, rs1),
        Lw { rd, rs1, offset } => format!("lw x{}, {}(x{})", rd, offset, rs1),
        Ld { rd, rs1, offset } => format!("ld x{}, {}(x{})", rd, offset, rs1),
        Lbu { rd, rs1, offset } => format!("lbu x{}, {}(x{})", rd, offset, rs1),
        Lhu { rd, rs1, offset } => format!("lhu x{}, {}(x{})", rd, offset, rs1),
        
        // Store instructions
        Sb { rs1, rs2, offset } => format!("sb x{}, {}(x{})", rs2, offset, rs1),
        Sh { rs1, rs2, offset } => format!("sh x{}, {}(x{})", rs2, offset, rs1),
        Sw { rs1, rs2, offset } => format!("sw x{}, {}(x{})", rs2, offset, rs1),
        
        // Branch instructions
        Beq { rs1, rs2, offset } => format!("beq x{}, x{}, {}", rs1, rs2, offset),
        Bne { rs1, rs2, offset } => format!("bne x{}, x{}, {}", rs1, rs2, offset),
        Blt { rs1, rs2, offset } => format!("blt x{}, x{}, {}", rs1, rs2, offset),
        Bge { rs1, rs2, offset } => format!("bge x{}, x{}, {}", rs1, rs2, offset),
        Bltu { rs1, rs2, offset } => format!("bltu x{}, x{}, {}", rs1, rs2, offset),
        Bgeu { rs1, rs2, offset } => format!("bgeu x{}, x{}, {}", rs1, rs2, offset),
        
        // Jump instructions
        Jal { rd, offset, compressed: _ } => format!("jal x{}, {}", rd, offset),
        Jalr { rd, rs1, offset, compressed: _ } => format!("jalr x{}, {}(x{})", rd, offset, rs1),
        
        // Upper immediate instructions
        Lui { rd, imm } => format!("lui x{}, 0x{:x}", rd, imm),
        Auipc { rd, imm } => format!("auipc x{}, 0x{:x}", rd, imm),
        
        // System instructions
        Ecall => "ecall".to_string(),
        Ebreak => "ebreak".to_string(),
        
        // Fence instructions
        Fence => "fence".to_string(),
        FenceI => "fence.i".to_string(),
        
        // M extension (multiply/divide)
        Mul { rd, rs1, rs2 } => format!("mul x{}, x{}, x{}", rd, rs1, rs2),
        Mulh { rd, rs1, rs2 } => format!("mulh x{}, x{}, x{}", rd, rs1, rs2),
        Mulhsu { rd, rs1, rs2 } => format!("mulhsu x{}, x{}, x{}", rd, rs1, rs2),
        Mulhu { rd, rs1, rs2 } => format!("mulhu x{}, x{}, x{}", rd, rs1, rs2),
        Div { rd, rs1, rs2 } => format!("div x{}, x{}, x{}", rd, rs1, rs2),
        Divu { rd, rs1, rs2 } => format!("divu x{}, x{}, x{}", rd, rs1, rs2),
        Rem { rd, rs1, rs2 } => format!("rem x{}, x{}, x{}", rd, rs1, rs2),
        Remu { rd, rs1, rs2 } => format!("remu x{}, x{}, x{}", rd, rs1, rs2),
        
        // A extension (atomic) - using actual names from the enum
        LrW { rd, rs1 } => format!("lr.w x{}, (x{})", rd, rs1),
        ScW { rd, rs1, rs2 } => format!("sc.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmoswapW { rd, rs1, rs2 } => format!("amoswap.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmoaddW { rd, rs1, rs2 } => format!("amoadd.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmoxorW { rd, rs1, rs2 } => format!("amoxor.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmoandW { rd, rs1, rs2 } => format!("amoand.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmoorW { rd, rs1, rs2 } => format!("amoor.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmominW { rd, rs1, rs2 } => format!("amomin.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmomaxW { rd, rs1, rs2 } => format!("amomax.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmominuW { rd, rs1, rs2 } => format!("amominu.w x{}, x{}, (x{})", rd, rs2, rs1),
        AmomaxuW { rd, rs1, rs2 } => format!("amomaxu.w x{}, x{}, (x{})", rd, rs2, rs1),
        
        // Special instructions
        MiscAlu { rd, rs2, op } => format!("{:?} x{}, x{}", op, rd, rs2),
        
        // Catch-all for any other instructions
        _ => format!("unknown instruction"),
    }
}