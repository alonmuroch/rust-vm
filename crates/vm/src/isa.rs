#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Load instructions: LW, LH, LB, etc.
    Load = 0x03,

    /// Store instructions: SW, SH, SB, etc.
    Store = 0x23,

    /// Branch instructions: BEQ, BNE, etc.
    Branch = 0x63,

    /// Jump and Link: Jumps to PC + imm, saves return address to `rd`
    Jal = 0x6f,

    /// Jump and Link Register: PC = rs1 + imm; rd = return address
    Jalr = 0x67,

    /// Immediate arithmetic ops: ADDI, SLTI, ANDI, ORI, etc.
    OpImm = 0x13,

    /// Register-register arithmetic ops: ADD, SUB, AND, OR, etc.
    Op = 0x33,

    /// Load Upper Immediate: Loads imm[31:12] << 12 into `rd`
    Lui = 0x37,

    /// Add Upper Immediate to PC: rd = PC + (imm << 12)
    Auipc = 0x17,

    /// System instructions: ECALL, EBREAK, CSR operations
    System = 0x73,

    /// Atomic Memory Operations: AMO instructions
    Amo = 0x2f,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        use Opcode::*;
        Some(match value {
            0x03 => Load,
            0x23 => Store,
            0x63 => Branch,
            0x6f => Jal,
            0x67 => Jalr,
            0x13 => OpImm,
            0x33 => Op,
            0x37 => Lui,
            0x17 => Auipc,
            0x73 => System,
            0x2f => Amo,
            _ => return None,
        })
    }
}


