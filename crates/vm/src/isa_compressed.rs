/// 3-bit compressed opcode space used for RV32C instruction decoding,
/// based on funct3 (bits 15–13) and the 2-bit base opcode (bits 1–0).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedOpcode {
    /// C.ADDI4SPN: Add unsigned immediate to stack pointer and write to rd′ (x8–x15)
    /// - Format: CIW
    /// - funct3 = 0b000, opcode = 0b00
    /// - Used in function prologues to quickly allocate stack space
    Addi4spn = 0x00,

    /// C.LW: Load a 32-bit word from memory into rd′
    /// - Format: CL
    /// - funct3 = 0b010, opcode = 0b00
    /// - Accesses memory at rs1′ + offset
    Lw = 0x01,

    /// C.SW: Store a 32-bit word from rs2′ into memory
    /// - Format: CS
    /// - funct3 = 0b110, opcode = 0b00
    /// - Stores word at address rs1′ + offset
    Sw = 0x02,

    /// C.ADDI / C.NOP: Add sign-extended immediate to register (or no-op if rd = x0 and imm = 0)
    /// - Format: CI
    /// - funct3 = 0b000, opcode = 0b01
    /// - C.NOP is a special case of C.ADDI (rd = x0, imm = 0)
    Addi = 0x03,

    /// C.JAL: Jump and link — jump relative to PC, store return address in x1 (RA)
    /// - Format: CJ
    /// - funct3 = 0b001, opcode = 0b01
    /// - Only supported in RV32
    Jal = 0x04,

    /// C.LI: Load immediate value into rd (sign-extended)
    /// - Format: CI
    /// - funct3 = 0b010, opcode = 0b01
    /// - Loads a 6-bit signed immediate
    Li = 0x05,

    /// C.LUI or C.ADDI16SP:
    /// - C.LUI: Load upper immediate into rd (rd ≠ x0 and x2)
    /// - C.ADDI16SP: Add signed immediate to x2 (SP), if rd = x2
    /// - Format: CI
    /// - funct3 = 0b011, opcode = 0b01
    LuiOrAddi16sp = 0x06,

    /// C.MISC-ALU:
    /// - Covers CB-format logic/shift immediate ops: C.SRLI, C.SRAI, C.ANDI
    /// - Covers CA-format register-register ops: C.SUB, C.XOR, C.OR, C.AND
    /// - Format: CB (immediates) and CA (register-register)
    /// - funct3 = 0b100, opcode = 0b01
    MiscAlu = 0x07,

    /// C.J: Jump unconditionally relative to PC
    /// - Format: CJ
    /// - funct3 = 0b101, opcode = 0b01
    /// - Similar to JAL, but no link (return address not saved)
    J = 0x08,

    /// C.BEQZ: Branch if rs1′ == 0
    /// - Format: CB
    /// - funct3 = 0b110, opcode = 0b01
    /// - 3-bit rs1′ selects x8–x15
    Beqz = 0x09,

    /// C.BNEZ: Branch if rs1′ ≠ 0
    /// - Format: CB
    /// - funct3 = 0b111, opcode = 0b01
    /// - 3-bit rs1′ selects x8–x15
    Bnez = 0x0A,

    /// C.SLLI: Shift rd left logically by immediate (zero-extended)
    /// - Format: CI
    /// - funct3 = 0b000, opcode = 0b10
    /// - Only legal if rd ≠ x0
    Slli = 0x0B,

    /// C.LWSP: Load a 32-bit word from SP-relative offset
    /// - Format: CI
    /// - funct3 = 0b010, opcode = 0b10
    /// - Offset is multiple of 4
    Lwsp = 0x0C,

    /// C.MV, C.ADD, C.JR, C.JALR, C.EBREAK, C.RET:
    /// - Format: CR
    /// - funct3 = 0b100, opcode = 0b10
    /// - Decoding depends on rs1, rs2:
    ///   - rs2 ≠ 0: MV (rd ← rs2) or ADD (rd ← rd + rs2)
    ///   - rs2 = 0: JR (x0 ← rs1) or JALR (x1 ← rs1)
    ///   - rs1 = x0 and rs2 = 0: EBREAK
    RegOrJump = 0x0D,

    /// C.SWSP: Store 32-bit word to SP-relative offset
    /// - Format: CSS
    /// - funct3 = 0b110, opcode = 0b10
    /// - Offset is multiple of 4
    Swsp = 0x0E,
}

impl CompressedOpcode {
    /// Decodes compressed opcode class from funct3 and 2-bit opcode.
    pub fn from_bits(funct3: u16, opcode: u16) -> Option<Self> {
        match (funct3, opcode) {
            (0b000, 0b00) => Some(Self::Addi4spn),
            (0b010, 0b00) => Some(Self::Lw),
            (0b110, 0b00) => Some(Self::Sw),

            (0b000, 0b01) => Some(Self::Addi),
            (0b001, 0b01) => Some(Self::Jal),
            (0b010, 0b01) => Some(Self::Li),
            (0b011, 0b01) => Some(Self::LuiOrAddi16sp),
            (0b100, 0b01) => Some(Self::MiscAlu),
            (0b101, 0b01) => Some(Self::J),
            (0b110, 0b01) => Some(Self::Beqz),
            (0b111, 0b01) => Some(Self::Bnez),

            (0b000, 0b10) => Some(Self::Slli),
            (0b010, 0b10) => Some(Self::Lwsp),
            (0b100, 0b10) => Some(Self::RegOrJump),
            (0b110, 0b10) => Some(Self::Swsp),

            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
