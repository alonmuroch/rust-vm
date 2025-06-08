/// This enum defines the full instruction set for RV32IMAC
/// (Integer, Multiplication, Atomic, and Compressed extensions).
/// Each variant corresponds to a decoded instruction
/// from either a 32-bit or 16-bit RISC-V word.
#[derive(Debug)]
pub enum Instruction {
    // ===== RV32I =====

    /// Add: rd = rs1 + rs2
    Add { rd: usize, rs1: usize, rs2: usize },
    /// Subtract: rd = rs1 - rs2
    Sub { rd: usize, rs1: usize, rs2: usize },
    /// Add immediate: rd = rs1 + imm
    Addi { rd: usize, rs1: usize, imm: i32 },

    /// AND: rd = rs1 & rs2
    And { rd: usize, rs1: usize, rs2: usize },
    /// OR: rd = rs1 | rs2
    Or { rd: usize, rs1: usize, rs2: usize },
    /// XOR: rd = rs1 ^ rs2
    Xor { rd: usize, rs1: usize, rs2: usize },

    /// ANDI: rd = rs1 & imm
    Andi { rd: usize, rs1: usize, imm: i32 },
    /// ORI: rd = rs1 | imm
    Ori { rd: usize, rs1: usize, imm: i32 },
    /// XORI: rd = rs1 ^ imm
    Xori { rd: usize, rs1: usize, imm: i32 },

    /// Set less than: rd = (rs1 < rs2)
    Slt { rd: usize, rs1: usize, rs2: usize },
    /// Set less than immediate: rd = (rs1 < imm)
    Slti { rd: usize, rs1: usize, imm: i32 },

    /// Shift left logical: rd = rs1 << rs2
    Sll { rd: usize, rs1: usize, rs2: usize },
    /// Shift right logical: rd = rs1 >> rs2 (logical)
    Srl { rd: usize, rs1: usize, rs2: usize },
    /// Shift right arithmetic: rd = rs1 >> rs2 (arithmetic)
    Sra { rd: usize, rs1: usize, rs2: usize },

    /// Shift left immediate: rd = rs1 << shamt
    Slli { rd: usize, rs1: usize, shamt: u8 },
    /// Shift right logical immediate: rd = rs1 >> shamt (logical)
    Srli { rd: usize, rs1: usize, shamt: u8 },
    /// Shift right arithmetic immediate: rd = rs1 >> shamt (arithmetic)
    Srai { rd: usize, rs1: usize, shamt: u8 },

    /// Load word: rd = *(rs1 + offset)
    Lw { rd: usize, rs1: usize, offset: i32 },
    /// Store word: *(rs1 + offset) = rs2
    Sw { rs1: usize, rs2: usize, offset: i32 },

    /// Branch if equal: if (rs1 == rs2) pc += offset
    Beq { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if not equal
    Bne { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if less than
    Blt { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if greater or equal
    Bge { rs1: usize, rs2: usize, offset: i32 },

    /// Jump and link: rd = pc + 4; pc += offset
    Jal { rd: usize, offset: i32 },
    /// Jump and link register: pc = rs1 + imm; rd = return address
    Jalr { rd: usize, rs1: usize, offset: i32 },

    /// Load upper immediate: rd = imm << 12
    Lui { rd: usize, imm: i32 },
    /// Add upper immediate to PC: rd = pc + (imm << 12)
    Auipc { rd: usize, imm: i32 },

    /// Environment call (used for syscall, halting, etc.)
    Ecall,

    // ===== RV32M =====

    /// Multiply: rd = rs1 * rs2
    Mul { rd: usize, rs1: usize, rs2: usize },
    /// Multiply high signed: rd = (rs1 * rs2)[63:32]
    Mulh { rd: usize, rs1: usize, rs2: usize },
    /// Multiply high unsigned
    Mulhu { rd: usize, rs1: usize, rs2: usize },
    /// Multiply signed * unsigned
    Mulhsu { rd: usize, rs1: usize, rs2: usize },

    /// Divide: rd = rs1 / rs2 (signed)
    Div { rd: usize, rs1: usize, rs2: usize },
    /// Divide unsigned
    Divu { rd: usize, rs1: usize, rs2: usize },
    /// Remainder
    Rem { rd: usize, rs1: usize, rs2: usize },
    /// Remainder unsigned
    Remu { rd: usize, rs1: usize, rs2: usize },

    // ===== RV32A (Atomics) =====

    /// Atomic swap (AMO): swap memory with register
    AmoswapW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic add
    AmoaddW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic bitwise AND
    AmoandW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic OR
    AmoorW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic XOR
    AmoxorW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic max signed
    AmomaxW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic min signed
    AmominW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic max unsigned
    AmomaxuW { rd: usize, rs1: usize, rs2: usize },
    /// Atomic min unsigned
    AmominuW { rd: usize, rs1: usize, rs2: usize },

    // ===== RV32C (Compressed Instructions) =====

    /// Jump to register (c.jr): pc = rs1
    Jr { rs1: usize },

    /// Return from function (c.ret): jr x1
    Ret,

    /// Move: rd = rs2 (c.mv)
    Mv { rd: usize, rs2: usize },

    /// Add immediate to stack pointer (c.addi16sp)
    Addi16sp { imm: i32 },

    /// No operation (c.nop): addi x0, x0, 0
    Nop,

    /// Branch if rs1 == 0 (c.beqz)
    Beqz { rs1: usize, offset: i32 },

    /// Branch if rs1 != 0 (c.bnez)
    Bnez { rs1: usize, offset: i32 },

    /// Environment breakpoint (compressed `C.EBREAK`)
    Ebreak,
}
