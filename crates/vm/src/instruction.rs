/// This enum defines the full instruction set for RV32IMAC
/// (Integer, Multiplication, Atomic, and Compressed extensions).
/// Each variant corresponds to a decoded instruction
/// from either a 32-bit or 16-bit RISC-V word.
/// 
/// EDUCATIONAL PURPOSE: This enum represents the complete instruction set
/// that our RISC-V virtual machine can execute. Understanding this structure
/// helps students learn about:
/// - CPU instruction formats and encoding
/// - Different types of operations (arithmetic, memory, control flow)
/// - How high-level programming constructs map to machine instructions
/// 
/// RISC-V INSTRUCTION FORMATS:
/// - R-type: Register operations (add, sub, and, or, etc.)
/// - I-type: Immediate operations (addi, lw, jalr, etc.)
/// - S-type: Store operations (sw, sh, sb)
/// - B-type: Branch operations (beq, bne, blt, etc.)
/// - U-type: Upper immediate operations (lui, auipc)
/// - J-type: Jump operations (jal)
/// 
/// INSTRUCTION CATEGORIES:
/// - ARITHMETIC: Basic math operations (add, sub, mul, div)
/// - LOGICAL: Bit manipulation (and, or, xor, shifts)
/// - MEMORY: Load and store operations
/// - CONTROL FLOW: Branches and jumps
/// - SYSTEM: Environment calls and breakpoints
/// - ATOMIC: Thread-safe memory operations
/// - COMPRESSED: 16-bit versions of common instructions
/// 
/// REAL-WORLD CONTEXT: In actual RISC-V processors, these instructions
/// are encoded as binary data. The decoder converts these binary patterns
/// into this enum structure, which makes them easier to work with in our VM.
/// 
/// PERFORMANCE IMPLICATIONS: Different instruction types have different
/// execution costs. Memory operations are typically slower than register
/// operations, and branches can cause pipeline stalls in real CPUs.
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
    /// Set less than unsigned: rd = (rs1 < rs2) as u32
    Sltu { rd: usize, rs1: usize, rs2: usize },
    /// Set less than immediate: rd = (rs1 < imm)
    Slti { rd: usize, rs1: usize, imm: i32 },
    /// Set Less Than Immediate Unsigned: rd = (rs1 < imm) ? 1 : 0
    Sltiu { rd: usize, rs1: usize, imm: i32 },

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
    /// Load Byte Unsigned: rd = zero_extend(memory[rs1 + offset])
    Lbu { rd: usize, rs1: usize, offset: i32 },
    /// Load Halfword: rd = sign_extend(*(rs1 + offset) as i16)
    Lh { rd: usize, rs1: usize, offset: i32 },
    /// Store halfword: *(rs1 + offset) = rs2 & 0xFFFF
    Sh { rs1: usize, rs2: usize, offset: i32 },
    /// Store word: *(rs1 + offset) = rs2
    Sw { rs1: usize, rs2: usize, offset: i32 },
    /// store byte
    Sb { rs1: usize, rs2: usize, offset: i32 },

    /// Branch if equal: if (rs1 == rs2) pc += offset
    Beq { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if not equal
    Bne { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if less than
    Blt { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if greater or equal
    Bge { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if less than (unsigned)
    Bltu { rs1: usize, rs2: usize, offset: i32 },
    /// Branch if greater or equal (unsigned)
    Bgeu { rs1: usize, rs2: usize, offset: i32 },

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

    /// Compressed add immediate to stack pointer + register
    Addi4spn { rd: usize, imm: u32 },

    /// No operation (c.nop): addi x0, x0, 0
    Nop,

    /// Branch if rs1 == 0 (c.beqz)
    Beqz { rs1: usize, offset: i32 },

    /// Branch if rs1 != 0 (c.bnez)
    Bnez { rs1: usize, offset: i32 },

    /// Environment breakpoint (compressed `C.EBREAK`)
    Ebreak,

    /// Compressed Miscellaneous ALU (C.SUB, C.XOR, C.OR, C.AND)
    MiscAlu { rd: usize, rs2: usize, op: MiscAluOp },
}

#[derive(Debug)]
pub enum MiscAluOp {
    Sub,
    Xor,
    Or,
    And,
}

impl Instruction {
    pub fn pretty_print(&self) -> String {
        fn reg(r: usize) -> String {
            format!("x{}", r) // or use register aliases like a0, t1, etc. if desired
        }

        match self {
            Instruction::Add { rd, rs1, rs2 } =>
                format!("add  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Sub { rd, rs1, rs2 } =>
                format!("sub  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Addi { rd, rs1, imm } =>
                format!("addi {}, {}, {}", reg(*rd), reg(*rs1), imm),

            Instruction::And { rd, rs1, rs2 } =>
                format!("and  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Or  { rd, rs1, rs2 } =>
                format!("or   {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Xor { rd, rs1, rs2 } =>
                format!("xor  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),

            Instruction::Andi { rd, rs1, imm } =>
                format!("andi {}, {}, {}", reg(*rd), reg(*rs1), imm),
            Instruction::Ori  { rd, rs1, imm } =>
                format!("ori  {}, {}, {}", reg(*rd), reg(*rs1), imm),
            Instruction::Xori { rd, rs1, imm } =>
                format!("xori {}, {}, {}", reg(*rd), reg(*rs1), imm),

            Instruction::Slt  { rd, rs1, rs2 } =>
                format!("slt  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Sltu { rd, rs1, rs2 } =>
                format!("sltu {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Slti { rd, rs1, imm } =>
                format!("slti {}, {}, {}", reg(*rd), reg(*rs1), imm),
            Instruction::Sltiu { rd, rs1, imm } =>
                format!("sltiu {}, {}, {}", reg(*rd), reg(*rs1), imm),

            Instruction::Sll  { rd, rs1, rs2 } =>
                format!("sll  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Srl  { rd, rs1, rs2 } =>
                format!("srl  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Sra  { rd, rs1, rs2 } =>
                format!("sra  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Slli { rd, rs1, shamt } =>
                format!("slli {}, {}, {}", reg(*rd), reg(*rs1), shamt),
            Instruction::Srli { rd, rs1, shamt } =>
                format!("srli {}, {}, {}", reg(*rd), reg(*rs1), shamt),
            Instruction::Srai { rd, rs1, shamt } =>
                format!("srai {}, {}, {}", reg(*rd), reg(*rs1), shamt),

            Instruction::Lw { rd, rs1, offset } =>
                format!("lw   {}, {}({})", reg(*rd), offset, reg(*rs1)),
            Instruction::Lbu { rd, rs1, offset } =>
                format!("lbu   {}, {}({})", reg(*rd), offset, reg(*rs1)),
            Instruction::Lh { rd, rs1, offset } =>
                format!("lh   {}, {}({})", reg(*rd), offset, reg(*rs1)),
            Instruction::Sh { rs1, rs2, offset } =>
                format!("sh   {}, {}({})", reg(*rs2), offset, reg(*rs1)),
            Instruction::Sw { rs1, rs2, offset } =>
                format!("sw   {}, {}({})", reg(*rs2), offset, reg(*rs1)),
            Instruction::Sb { rs1, rs2, offset } =>
                format!("sb   {}, {}({})", reg(*rs2), offset, reg(*rs1)),

            Instruction::Beq { rs1, rs2, offset } =>
                format!("beq  {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),
            Instruction::Bne { rs1, rs2, offset } =>
                format!("bne  {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),
            Instruction::Blt { rs1, rs2, offset } =>
                format!("blt  {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),
            Instruction::Bge { rs1, rs2, offset } =>
                format!("bge  {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),
            Instruction::Bltu { rs1, rs2, offset } =>
                format!("bltu {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),
            Instruction::Bgeu { rs1, rs2, offset } =>
                format!("bgeu {}, {}, pc+{}", reg(*rs1), reg(*rs2), offset),

            Instruction::Jal { rd, offset } =>
                format!("jal  {}, pc+{}", reg(*rd), offset),
            Instruction::Jalr { rd, rs1, offset } =>
                format!("jalr {}, {}({})", reg(*rd), offset, reg(*rs1)),

            Instruction::Lui { rd, imm } =>
                format!("lui  {}, {}", reg(*rd), imm),
            Instruction::Auipc { rd, imm } =>
                format!("auipc {}, {}", reg(*rd), imm),

            Instruction::Ecall =>
                "ecall".to_string(),

            Instruction::Mul { rd, rs1, rs2 } =>
                format!("mul  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Mulh { rd, rs1, rs2 } =>
                format!("mulh {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Mulhsu { rd, rs1, rs2 } =>
                format!("mulhsu {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Mulhu { rd, rs1, rs2 } =>
                format!("mulhu {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),

            Instruction::Div { rd, rs1, rs2 } =>
                format!("div  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Divu { rd, rs1, rs2 } =>
                format!("divu {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Rem { rd, rs1, rs2 } =>
                format!("rem  {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::Remu { rd, rs1, rs2 } =>
                format!("remu {}, {}, {}", reg(*rd), reg(*rs1), reg(*rs2)),

            Instruction::AmoswapW { rd, rs1, rs2 } =>
                format!("amoswap.w {}, ({}) <- {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmoaddW { rd, rs1, rs2 } =>
                format!("amoadd.w  {}, ({}) + {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmoandW { rd, rs1, rs2 } =>
                format!("amoand.w  {}, ({}) & {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmoorW { rd, rs1, rs2 } =>
                format!("amoor.w   {}, ({}) | {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmoxorW { rd, rs1, rs2 } =>
                format!("amoxor.w  {}, ({}) ^ {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmomaxW { rd, rs1, rs2 } =>
                format!("amomax.w  {}, ({}) max {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmominW { rd, rs1, rs2 } =>
                format!("amomin.w  {}, ({}) min {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmomaxuW { rd, rs1, rs2 } =>
                format!("amomaxu.w {}, ({}) maxu {}", reg(*rd), reg(*rs1), reg(*rs2)),
            Instruction::AmominuW { rd, rs1, rs2 } =>
                format!("amominu.w {}, ({}) minu {}", reg(*rd), reg(*rs1), reg(*rs2)),

            Instruction::Jr { rs1 } =>
                format!("jr   {}", reg(*rs1)),
            Instruction::Ret =>
                "ret".to_string(),
            Instruction::Mv { rd, rs2 } =>
                format!("mv   {}, {}", reg(*rd), reg(*rs2)),
            Instruction::Addi16sp { imm } =>
                format!("addi16sp sp, {}", imm),
            Instruction::Addi4spn { rd, imm } =>
                format!("c.addi4spn {}, {}", reg(*rd), imm),
                
            Instruction::Nop =>
                "nop".to_string(),
            Instruction::Beqz { rs1, offset } =>
                format!("beqz {}, pc+{}", reg(*rs1), offset),
            Instruction::Bnez { rs1, offset } =>
                format!("bnez {}, pc+{}", reg(*rs1), offset),
            Instruction::Ebreak =>
                "ebreak".to_string(),

            Instruction::MiscAlu { rd, rs2, op } => {
                let op_str = match op {
                    MiscAluOp::Sub => "c.sub",
                    MiscAluOp::Xor => "c.xor",
                    MiscAluOp::Or  => "c.or",
                    MiscAluOp::And => "c.and",
                };
                format!("{} {}, {}", op_str, reg(*rd), reg(*rs2))
            }
        }
    }
}
