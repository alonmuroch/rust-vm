use crate::decoder::{decode_full, decode_compressed};
use crate::instruction::Instruction;
use crate::memory_page::MemoryPage;
use storage::Storage;
use std::rc::Rc;
use core::cell::RefCell;

/// Represents the Central Processing Unit (CPU) of our RISC-V virtual machine.
/// 
/// EDUCATIONAL PURPOSE: This struct models the core components of a real CPU:
/// - Program Counter (PC): Points to the next instruction to execute
/// - Registers: Fast storage locations for temporary data
/// - Verbose flag: For debugging and educational purposes
/// 
/// RISC-V ARCHITECTURE NOTES:
/// - RISC-V is a Reduced Instruction Set Computer (RISC) architecture
/// - It has 32 general-purpose registers (x0-x31)
/// - Register x0 is always hardwired to zero
/// - The PC is separate from the general registers
/// 
/// REAL CPU COMPARISON: This is a simplified model of a real CPU. In actual
/// hardware, a CPU has many more components:
/// - Arithmetic Logic Unit (ALU) for mathematical operations
/// - Control Unit for instruction decoding and control flow
/// - Cache memory for faster data access
/// - Pipeline stages for parallel instruction processing
/// - Memory Management Unit (MMU) for virtual memory
/// 
/// VIRTUAL MACHINE CONTEXT: In a VM, we simulate these components in software.
/// This allows us to run programs written for one architecture (RISC-V) on
/// different hardware (like x86 or ARM). The VM provides an abstraction layer
/// that makes the underlying hardware details transparent to the running program.
/// 
/// MEMORY MANAGEMENT: We use Rc<RefCell<>> for shared mutable access to memory
/// and storage, which allows the CPU to read/write memory while maintaining
/// Rust's safety guarantees.
/// 
/// PERFORMANCE CONSIDERATIONS: This is an interpretive VM, meaning each
/// instruction is decoded and executed one at a time. Real CPUs use techniques
/// like pipelining, out-of-order execution, and just-in-time compilation to
/// achieve much higher performance. However, this simple approach is perfect
/// for learning and understanding how CPUs work at a fundamental level.
#[derive(Debug)]
pub struct CPU {
    /// Program Counter - points to the next instruction to execute
    /// EDUCATIONAL: In real CPUs, this is a special register that automatically
    /// increments after each instruction (unless the instruction changes it)
    pub pc: u32,
    
    /// General-purpose registers (x0-x31)
    /// EDUCATIONAL: Registers are the fastest storage in a CPU, much faster than
    /// main memory. RISC-V has 32 registers, each holding a 32-bit value.
    /// Register x0 is always zero, x1 is the return address, x2 is the stack pointer.
    pub regs: [u32; 32],

    /// Enable verbose logging for debugging and educational purposes
    /// EDUCATIONAL: This helps students understand what the CPU is doing
    /// by printing each instruction as it executes
    pub verbose: bool,
}

impl CPU {
    /// Creates a new CPU instance with default values.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates CPU initialization. In real systems,
    /// the CPU would be reset to a known state when powered on.
    /// 
    /// INITIALIZATION DETAILS:
    /// - PC starts at 0 (first instruction)
    /// - All registers start at 0 (except x0 which is always 0)
    /// - Verbose logging is disabled by default
    pub fn new() -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            verbose: false,
        }
    }

    /// Executes a single instruction cycle (fetch, decode, execute).
    /// 
    /// EDUCATIONAL PURPOSE: This is the heart of the CPU - the instruction cycle.
    /// Every CPU follows this basic pattern:
    /// 1. Fetch: Read the next instruction from memory
    /// 2. Decode: Figure out what the instruction does
    /// 3. Execute: Perform the operation
    /// 
    /// INSTRUCTION CYCLE DETAILS:
    /// - FETCH: The CPU reads the instruction from memory at the address
    ///   pointed to by the Program Counter (PC)
    /// - DECODE: The instruction is analyzed to determine what operation
    ///   to perform and what operands (registers, memory addresses) to use
    /// - EXECUTE: The actual operation is performed (arithmetic, memory access,
    ///   control flow, etc.)
    /// 
    /// ERROR HANDLING: If an invalid instruction is encountered, the CPU
    /// handles it gracefully by calling unknown_instruction() which provides
    /// debugging information and halts execution safely.
    /// 
    /// RETURN VALUE: Returns true if execution should continue, false to halt
    /// 
    /// MEMORY ACCESS: Uses shared references to memory and storage to allow
    /// the CPU to read/write while maintaining Rust's safety guarantees.
    /// 
    /// REAL-WORLD ANALOGY: This is like a factory assembly line where each
    /// worker (instruction) performs a specific task. The conveyor belt (PC)
    /// moves to the next task automatically, unless a task specifically
    /// redirects the flow (like a branch or jump instruction).
    pub fn step(&mut self, memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> bool {
        // EDUCATIONAL: Step 1 - Fetch and decode the next instruction
        let instr = self.next_instruction(Rc::clone(&memory));
        
        // EDUCATIONAL: Step 2 - Execute the instruction or handle errors
        match instr {
            Some((instr, size)) => {
                // Valid instruction found - execute it
                self.run_instruction(instr, size, Rc::clone(&memory), storage)
            }
            None => {
                // No valid instruction found - handle the error
                self.unknown_instruction(Rc::clone(&memory), storage)
            }
        }
    }

    /// Executes a single instruction and updates the program counter.
    /// 
    /// EDUCATIONAL PURPOSE: This function demonstrates instruction execution
    /// and program counter management. Some instructions (like branches and jumps)
    /// modify the PC directly, while others just increment it.
    /// 
    /// PC MANAGEMENT: The PC is only incremented if the instruction didn't
    /// change it. This handles branches, jumps, and calls correctly.
    /// 
    /// PARAMETERS:
    /// - instr: The decoded instruction to execute
    /// - size: Size of the instruction in bytes (2 for compressed, 4 for full)
    /// - memory: Shared reference to memory for load/store operations
    /// - storage: Shared reference to persistent storage
    fn run_instruction(&mut self, instr: Instruction, size: u8, memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> bool {
        // EDUCATIONAL: Debug output to help understand what's happening
        if self.verbose {
            println!("PC = 0x{:08x}, Instr = {}", self.pc, instr.pretty_print());
        }
        
        // EDUCATIONAL: Remember the old PC to detect if the instruction changed it
        let old_pc = self.pc;
        
        // EDUCATIONAL: Execute the instruction
        let result = self.execute(instr, memory, storage);      

        // EDUCATIONAL: Only increment PC if the instruction didn't change it
        // This handles branches, jumps, and calls correctly
        if self.pc == old_pc {
            self.pc = self.pc.wrapping_add(size as u32);
        }
        result
    }

    /// Handles unknown or invalid instructions.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates error handling in CPU design.
    /// When a CPU encounters an invalid instruction, it needs to handle it
    /// gracefully rather than crashing.
    /// 
    /// DEBUGGING: This function provides detailed information about what
    /// went wrong, including the hex dump of the invalid bytes.
    /// 
    /// RETURN VALUE: Returns false to halt execution on invalid instructions
    fn unknown_instruction(&mut self, memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> bool {
        // EDUCATIONAL: Try to read the invalid instruction bytes for debugging
        if let Some(slice_ref) = memory.borrow().mem_slice(self.pc as usize, self.pc as usize + 4) {
            // EDUCATIONAL: Convert bytes to hex for human-readable debugging
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

    /// Fetches and decodes the next instruction from memory.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates the fetch and decode phases
    /// of the instruction cycle. It handles both regular (32-bit) and
    /// compressed (16-bit) RISC-V instructions.
    /// 
    /// RISC-V COMPRESSED INSTRUCTIONS: RISC-V supports 16-bit compressed
    /// instructions to reduce code size. The bottom 2 bits determine if
    /// an instruction is compressed (not 0b11) or regular (0b11).
    /// 
    /// RETURN VALUE: Returns Some((instruction, size)) if successful, None if invalid
    pub fn next_instruction(&mut self, memory: Rc<RefCell<MemoryPage>>) -> Option<(Instruction, u8)> {
        let pc = self.pc as usize;
        let mem_ref = memory.borrow();
        
        // EDUCATIONAL: Read 4 bytes from memory (enough for any instruction)
        let bytes = mem_ref.mem_slice(pc, pc + 4)?;

        // EDUCATIONAL: Need at least 2 bytes for any instruction
        if bytes.len() < 2 {
            return None;
        }

        // EDUCATIONAL: Check if this is a compressed instruction
        // RISC-V compressed instructions have bottom 2 bits != 0b11
        let hword = u16::from_le_bytes([bytes[0], bytes[1]]);
        let is_compressed = (hword & 0b11) != 0b11;

        if is_compressed {
            // EDUCATIONAL: Decode 16-bit compressed instruction
            decode_compressed(hword).map(|inst| (inst, 2))
        } else if bytes.len() >= 4 {
            // EDUCATIONAL: Decode 32-bit regular instruction
            let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            decode_full(word).map(|inst| (inst, 4))
        } else {
            None
        }
    }

    /// Executes a decoded instruction.
    /// 
    /// EDUCATIONAL PURPOSE: This is the execute phase of the instruction cycle.
    /// It contains the implementation of all RISC-V instructions supported by
    /// our VM. This is where the actual computation happens.
    /// 
    /// INSTRUCTION CATEGORIES:
    /// - Arithmetic: ADD, SUB, MUL, DIV, etc.
    /// - Logical: AND, OR, XOR, shifts
    /// - Memory: Load and store operations
    /// - Control: Branches and jumps
    /// - System: System calls and special operations
    /// 
    /// REGISTER CONVENTIONS:
    /// - rd: Destination register (where result goes)
    /// - rs1, rs2: Source registers (operands)
    /// - imm: Immediate value (constant)
    /// 
    /// RETURN VALUE: Returns true to continue execution, false to halt
    pub fn execute(&mut self, instr: Instruction, memory: Rc<RefCell<MemoryPage>>, storage: Rc<RefCell<Storage>>) -> bool {
        match instr {
            // EDUCATIONAL: Arithmetic instructions - perform mathematical operations
            Instruction::Add { rd, rs1, rs2 } => {
                // EDUCATIONAL: Use wrapping_add to handle overflow correctly
                // In real CPUs, overflow might set flags or cause exceptions
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2])
            }
            Instruction::Sub { rd, rs1, rs2 } => {
                self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2])
            }
            Instruction::Addi { rd, rs1, imm } => {
                // EDUCATIONAL: Add immediate - adds a constant to a register
                // This is very common in real programs
                self.regs[rd] = self.regs[rs1].wrapping_add(imm as u32)
            }
            
            // EDUCATIONAL: Logical instructions - perform bitwise operations
            Instruction::And { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] & self.regs[rs2],
            Instruction::Or { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] | self.regs[rs2],
            Instruction::Xor { rd, rs1, rs2 } => self.regs[rd] = self.regs[rs1] ^ self.regs[rs2],
            Instruction::Andi { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] & (imm as u32),
            Instruction::Ori { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] | (imm as u32),
            Instruction::Xori { rd, rs1, imm } => self.regs[rd] = self.regs[rs1] ^ (imm as u32),
            
            // EDUCATIONAL: Comparison instructions - set result to 0 or 1
            Instruction::Slt { rd, rs1, rs2 } => {
                // EDUCATIONAL: Set if less than (signed comparison)
                self.regs[rd] = (self.regs[rs1] as i32).lt(&(self.regs[rs2] as i32)) as u32
            }
            Instruction::Sltu { rd, rs1, rs2 } => {
                // EDUCATIONAL: Set if less than (unsigned comparison)
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
            
            // EDUCATIONAL: Shift instructions - move bits left or right
            Instruction::Sll { rd, rs1, rs2 } => {
                // EDUCATIONAL: Logical left shift - multiply by 2^shift_amount
                // The & 0x1F ensures shift amount is 0-31 (5 bits)
                self.regs[rd] = self.regs[rs1] << (self.regs[rs2] & 0x1F)
            }
            Instruction::Srl { rd, rs1, rs2 } => {
                // EDUCATIONAL: Logical right shift - divide by 2^shift_amount
                self.regs[rd] = self.regs[rs1] >> (self.regs[rs2] & 0x1F)
            }
            Instruction::Sra { rd, rs1, rs2 } => {
                // EDUCATIONAL: Arithmetic right shift - preserves sign bit
                self.regs[rd] = ((self.regs[rs1] as i32) >> (self.regs[rs2] & 0x1F)) as u32
            }
            Instruction::Slli { rd, rs1, shamt } => self.regs[rd] = self.regs[rs1] << shamt,
            Instruction::Srli { rd, rs1, shamt } => self.regs[rd] = self.regs[rs1] >> shamt,
            Instruction::Srai { rd, rs1, shamt } => {
                self.regs[rd] = ((self.regs[rs1] as i32) >> shamt) as u32
            }
            
            // EDUCATIONAL: Load instructions - read data from memory into registers
            Instruction::Lw { rd, rs1, offset } => {
                // EDUCATIONAL: Load word (32-bit) from memory
                // Address = base register + offset
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                self.regs[rd] = memory.borrow().load_u32(addr);
            }
            Instruction::Lbu { rd, rs1, offset } => {
                // EDUCATIONAL: Load byte unsigned (8-bit, zero-extended)
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                let byte = memory.borrow().load_byte(addr);
                self.regs[rd] = byte as u32;
            }
            Instruction::Lh { rd, rs1, offset } => {
                // EDUCATIONAL: Load halfword (16-bit, sign-extended)
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                let halfword = memory.borrow().load_halfword(addr); // returns u16
                let value = (halfword as i16) as i32 as u32; // sign-extend to 32-bit
                self.regs[rd] = value;
            }

            // EDUCATIONAL: Store instructions - write data from registers to memory
            Instruction::Sh { rs1, rs2, offset } => {
                // EDUCATIONAL: Store halfword (16-bit)
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.borrow_mut().store_u16(addr, (self.regs[rs2] & 0xFFFF) as u16);
            }
            Instruction::Sw { rs1, rs2, offset } => {
                // EDUCATIONAL: Store word (32-bit)
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.borrow_mut().store_u32(addr, self.regs[rs2]);
            }
            Instruction::Sb { rs1, rs2, offset } => {
                // EDUCATIONAL: Store byte (8-bit)
                let addr = self.regs[rs1].wrapping_add(offset as u32) as usize;
                memory.borrow_mut().store_u8(addr, (self.regs[rs2] & 0xFF) as u8);
            }
        
            // EDUCATIONAL: Branch instructions - conditionally change the PC
            // These implement if/else and loop constructs
            Instruction::Beq { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if equal - jump if two registers are equal
                if self.regs[rs1] == self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bne { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if not equal
                if self.regs[rs1] != self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Blt { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if less than (signed comparison)
                if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bge { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if greater than or equal (signed)
                if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bltu { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if less than (unsigned comparison)
                if self.regs[rs1] < self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }

            Instruction::Bgeu { rs1, rs2, offset } => {
                // EDUCATIONAL: Branch if greater than or equal (unsigned)
                if self.regs[rs1] >= self.regs[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            // EDUCATIONAL: Jump and Link instructions - for function calls
            Instruction::Jal { rd, offset } => {
                // EDUCATIONAL: JAL (Jump and Link) - used for function calls
                // Saves the return address (PC + 4) in rd, then jumps to PC + offset
                // Note: rd = 0 is special (x0 is always zero), so we don't write to it
                if rd != 0 {
                    self.regs[rd] = self.pc + 4;
                }
                self.pc = self.pc.wrapping_add(offset as u32);
                return true;
            }
            Instruction::Jalr { rd, rs1, offset } => {
                // EDUCATIONAL: JALR (Jump and Link Register) - indirect function calls
                // Target address = base register + offset, with bottom bit cleared
                // This ensures proper alignment and is required by RISC-V spec
                let base = self.regs[rs1];
                let target = base.wrapping_add(offset as u32) & !1;
                let return_address = self.pc + 4;

                if rd != 0 {
                    self.regs[rd] = return_address;
                } 

                self.pc = target;
                return true;
            }

            // EDUCATIONAL: Load Upper Immediate - loads immediate into upper bits
            Instruction::Lui { rd, imm } => {
                // EDUCATIONAL: LUI loads a 20-bit immediate into bits 31-12 of rd
                // This is used to load large constants (like addresses) into registers
                self.regs[rd] = (imm << 12) as u32
            }
            Instruction::Auipc { rd, imm } => {
                // EDUCATIONAL: AUIPC (Add Upper Immediate to PC) - PC-relative addressing
                // Used for position-independent code and loading addresses relative to PC
                self.regs[rd] = self.pc.wrapping_add((imm << 12) as u32);
            }
            
            // EDUCATIONAL: Multiplication instructions - extended arithmetic
            Instruction::Mul { rd, rs1, rs2 } => {
                // EDUCATIONAL: MUL - multiply two registers, store lower 32 bits
                self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2])
            }
            Instruction::Mulh { rd, rs1, rs2 } => {
                // EDUCATIONAL: MULH - multiply signed, store upper 32 bits
                self.regs[rd] = (((self.regs[rs1] as i64) * (self.regs[rs2] as i64)) >> 32) as u32
            }
            Instruction::Mulhu { rd, rs1, rs2 } => {
                // EDUCATIONAL: MULHU - multiply unsigned, store upper 32 bits
                self.regs[rd] = (((self.regs[rs1] as u64) * (self.regs[rs2] as u64)) >> 32) as u32
            }
            Instruction::Mulhsu { rd, rs1, rs2 } => {
                // EDUCATIONAL: MULHSU - multiply signed by unsigned, store upper 32 bits
                self.regs[rd] =
                    (((self.regs[rs1] as i64) * (self.regs[rs2] as u64 as i64)) >> 32) as u32
            }
            // EDUCATIONAL: Division and remainder instructions
            Instruction::Div { rd, rs1, rs2 } => {
                // EDUCATIONAL: DIV - signed division with wrapping behavior
                self.regs[rd] = (self.regs[rs1] as i32).wrapping_div(self.regs[rs2] as i32) as u32
            }
            Instruction::Divu { rd, rs1, rs2 } => {
                // EDUCATIONAL: DIVU - unsigned division
                self.regs[rd] = self.regs[rs1] / self.regs[rs2]
            }
            Instruction::Rem { rd, rs1, rs2 } => {
                // EDUCATIONAL: REM - signed remainder with wrapping behavior
                self.regs[rd] = (self.regs[rs1] as i32).wrapping_rem(self.regs[rs2] as i32) as u32
            }
            Instruction::Remu { rd, rs1, rs2 } => {
                // EDUCATIONAL: REMU - unsigned remainder
                self.regs[rd] = self.regs[rs1] % self.regs[rs2]
            }
            
            // EDUCATIONAL: System instructions - for OS interaction and debugging
            Instruction::Ecall => {
                // EDUCATIONAL: ECALL - Environment Call - triggers a system call
                // This is how user programs request services from the operating system
                return self.handle_syscall(memory, storage);
            }
            Instruction::Ebreak => {
                // EDUCATIONAL: EBREAK - Environment Break - for debugging
                // In real systems, this would trigger a debugger breakpoint
                return false
            }
            
            // EDUCATIONAL: Compressed instruction set (RV32C) - space-saving instructions
            Instruction::Jr { rs1 } => {
                // EDUCATIONAL: JR (Jump Register) - compressed jump to register
                self.pc = self.regs[rs1];
                return true;
            }
            Instruction::Ret => {
                // EDUCATIONAL: RET - compressed return instruction
                // Equivalent to JR x1 (jump to return address register)
                let target = self.regs[1]; // x1 = ra (return address)
                if target == 0 || target == 0xFFFF_FFFF {
                    return false; // halt if ret target is 0 or invalid
                }
    
                self.pc = target;
                return true;
            }
            Instruction::Mv { rd, rs2 } => {
                // EDUCATIONAL: MV (Move) - compressed register copy
                self.regs[rd] = self.regs[rs2]
            }
            Instruction::Addi16sp { imm } => {
                // EDUCATIONAL: ADDI16SP - add immediate to stack pointer
                // x2 is the stack pointer (SP)
                self.regs[2] = self.regs[2].wrapping_add(imm as u32)
            }
            Instruction::Addi4spn { rd, imm } => {
                // EDUCATIONAL: ADDI4SPN - add immediate to SP, store in rd
                // Used for stack frame setup in function prologues
                self.regs[rd] = self.regs[2].wrapping_add(imm);
            }
            Instruction::Nop => {
                // EDUCATIONAL: NOP - No Operation - does nothing
                // Used for alignment and timing in real systems
            }
            Instruction::Beqz { rs1, offset } => {
                // EDUCATIONAL: BEQZ - Branch if Equal to Zero (compressed)
                if self.regs[rs1] == 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }
            Instruction::Bnez { rs1, offset } => {
                // EDUCATIONAL: BNEZ - Branch if Not Equal to Zero (compressed)
                if self.regs[rs1] != 0 {
                    self.pc = self.pc.wrapping_add(offset as u32);
                    return true;
                }
            }

            // EDUCATIONAL: Miscellaneous ALU operations (compressed)
            Instruction::MiscAlu { rd, rs2, op } => {
                match op {
                    crate::instruction::MiscAluOp::Sub => {
                        // EDUCATIONAL: C.SUB - compressed subtract
                        self.regs[rd] = self.regs[rd].wrapping_sub(self.regs[rs2]);
                    }
                    crate::instruction::MiscAluOp::Xor => {
                        // EDUCATIONAL: C.XOR - compressed XOR
                        self.regs[rd] = self.regs[rd] ^ self.regs[rs2];
                    }
                    crate::instruction::MiscAluOp::Or => {
                        // EDUCATIONAL: C.OR - compressed OR
                        self.regs[rd] = self.regs[rd] | self.regs[rs2];
                    }
                    crate::instruction::MiscAluOp::And => {
                        // EDUCATIONAL: C.AND - compressed AND
                        self.regs[rd] = self.regs[rd] & self.regs[rs2];
                    }
                }
            }
            _ => todo!("unhandled instruction"),
        }
        true

    }               
}
