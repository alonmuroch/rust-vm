use crate::cpu::CPU;
use crate::host_interface::HostInterface;
use crate::memory::Memory;
use crate::metering::Metering;
use crate::registers::Register;
use crate::sys_call::{DefaultSyscallHandler, SyscallHandler};
use core::cell::RefCell;
use std::rc::Rc;
use storage::Storage;

/// Represents a complete RISC-V virtual machine.
///
/// EDUCATIONAL PURPOSE: This struct encapsulates all the components needed
/// to run a virtual machine: CPU, memory, and persistent storage. It provides
/// a high-level interface for VM operations while hiding the complexity of
/// the underlying components.
///
/// VM ARCHITECTURE OVERVIEW:
/// - CPU: Executes RISC-V instructions
/// - Memory: Provides RAM for the running program
/// - Storage: Persistent storage for data that survives between runs
///
/// MEMORY MANAGEMENT: Uses Rc to share a trait-backed memory implementation
/// and Rc<RefCell<Storage>> for persistent storage, allowing the VM to manage
/// resources efficiently while maintaining Rust's safety guarantees.
#[derive(Debug)]
pub struct VM {
    /// The CPU that executes RISC-V instructions
    pub cpu: CPU,

    /// Shared reference to the VM's memory (RAM)
    pub memory: Memory,

    /// Shared reference to persistent storage
    pub storage: Rc<RefCell<Storage>>,

    pub host: Box<dyn HostInterface>,
}

impl VM {
    /// Creates a new virtual machine with the specified memory, storage, and host, using the default syscall handler.
    pub fn new(
        memory: Memory,
        storage: Rc<RefCell<Storage>>,
        host: Box<dyn HostInterface>,
    ) -> Self {
        Self::new_with_syscall_handler(
            memory,
            storage,
            host,
            Box::new(DefaultSyscallHandler::new()),
        )
    }

    /// Creates a new virtual machine with a writer for logging output.
    pub fn new_with_writer(
        memory: Memory,
        storage: Rc<RefCell<Storage>>,
        host: Box<dyn HostInterface>,
        writer: Option<Rc<RefCell<dyn core::fmt::Write>>>,
    ) -> Self {
        Self::new_with_syscall_handler(
            memory,
            storage,
            host,
            Box::new(DefaultSyscallHandler::with_writer(writer)),
        )
    }

    /// Creates a new virtual machine with a custom syscall handler.
    /// This is useful for testing or custom environments.
    pub fn new_with_syscall_handler(
        memory: Memory,
        storage: Rc<RefCell<Storage>>,
        host: Box<dyn HostInterface>,
        syscall_handler: Box<dyn SyscallHandler>,
    ) -> Self {
        let mut cpu = CPU::new(syscall_handler);
        cpu.regs[Register::Sp as usize] = memory.stack_top();
        Self {
            cpu,
            memory,
            storage,
            host,
        }
    }

    /// Installs a metering implementation on the underlying CPU.
    pub fn set_metering(&mut self, metering: Box<dyn Metering>) {
        self.cpu.set_metering(metering);
    }

    /// Loads program code into memory and sets the starting address.
    ///
    /// EDUCATIONAL PURPOSE: This demonstrates how programs are loaded into
    /// a VM. In real systems, this would involve loading from disk, parsing
    /// executable formats, and setting up memory protection.
    ///
    /// PARAMETERS:
    /// - alloc_add: The address where the code should be allocated in memory
    /// - start_addr: Where the program should start executing from
    /// - code: The binary program code to load
    ///
    /// MEMORY LAYOUT: Programs are typically loaded at specific addresses
    /// to ensure proper alignment and to avoid conflicts with system memory.
    pub fn set_code(&mut self, alloc_add: u32, start_addr: u32, code: &[u8]) {
        // EDUCATIONAL: Write the program code to memory starting at address 0
        self.memory.write_code(alloc_add as usize, code);

        // EDUCATIONAL: Set the program counter to the starting address
        self.cpu.pc = start_addr;
    }

    /// Allocates memory on the heap and writes data to it.
    ///
    /// EDUCATIONAL PURPOSE: This demonstrates dynamic memory allocation in a VM.
    /// Programs need to allocate memory for variables, arrays, and other data
    /// structures at runtime.
    ///
    /// HEAP MANAGEMENT: The VM maintains a heap pointer that moves forward
    /// as memory is allocated. This is a simple but effective allocation strategy.
    ///
    /// RETURN VALUE: Returns the address where the data was written
    pub fn alloc_and_write(&mut self, data: &[u8]) -> u32 {
        self.memory.alloc_on_heap(data)
    }

    /// Sets a register to point to data in memory.
    ///
    /// EDUCATIONAL PURPOSE: This demonstrates how to pass data to programs
    /// running in the VM. Instead of copying data into registers (which are
    /// limited in size), we store the data in memory and pass the address.
    ///
    /// PARAMETER PASSING: This is how we pass strings, arrays, and other
    /// large data structures to programs. The register contains a pointer
    /// to the actual data in memory.
    ///
    /// DEBUG OUTPUT: The function prints information about what it's doing,
    /// which is helpful for understanding VM behavior during development.
    pub fn set_reg_to_data(&mut self, reg: Register, data: &[u8]) -> u32 {
        // EDUCATIONAL: Allocate memory and write the data
        let addr = self.alloc_and_write(data);

        // EDUCATIONAL: Set the register to point to the data
        self.cpu.regs[reg as usize] = addr;

        // EDUCATIONAL: Debug output to help understand what's happening
        println!(
            "📥 set reg x{} to addr 0x{:08x} (len = {})",
            reg as u32,
            addr,
            data.len()
        );
        println!("📦 data written to 0x{:08x}: {:02x?}", addr, data);

        addr
    }

    /// Sets a register to a 32-bit value.
    ///
    /// EDUCATIONAL PURPOSE: This is used for passing small values (like
    /// integers) directly to programs. For larger data, use set_reg_to_data.
    ///
    /// USAGE: Typically used for passing function parameters, flags, or
    /// other small values that fit in a single register.
    pub fn set_reg_u32(&mut self, reg: Register, data: u32) {
        self.cpu.regs[reg as usize] = data;
    }

    /// Dumps the entire memory contents for debugging.
    ///
    /// EDUCATIONAL PURPOSE: This demonstrates memory inspection tools that
    /// are essential for debugging VM programs. It shows both hex and ASCII
    /// representations of memory contents.
    ///
    /// DEBUGGING: Memory dumps are crucial for understanding what's happening
    /// when programs don't work as expected. They show the actual data in memory.
    pub fn dump_all_memory(&self) {
        self.dump_memory(0, self.memory.mem().len());
    }

    /// Dumps a specific range of memory for debugging.
    ///
    /// EDUCATIONAL PURPOSE: This function provides a detailed view of memory
    /// contents, showing both hexadecimal and ASCII representations. This is
    /// similar to tools like 'hexdump' or 'xxd' in Unix systems.
    ///
    /// OUTPUT FORMAT:
    /// - Address in hexadecimal
    /// - 16 bytes of data in hex format
    /// - ASCII representation (printable characters only)
    /// - Heap pointer location
    ///
    /// MEMORY LAYOUT: The output shows how memory is organized, including
    /// where the heap pointer is and what data is stored where.
    pub fn dump_memory(&self, start: usize, end: usize) {
        let borrowed_memory = self.memory.as_ref();

        // EDUCATIONAL: Validate memory range to prevent errors
        assert!(start < end, "invalid memory range");
        assert!(end <= borrowed_memory.mem().len(), "range out of bounds");

        // EDUCATIONAL: Show heap pointer for context
        let next_heap = borrowed_memory.next_heap();
        println!("--- Memory Dump ---");
        println!("Next heap pointer: 0x{:08x}", next_heap);

        // EDUCATIONAL: Display memory in 16-byte lines
        for addr in (start..end).step_by(16) {
            let line = &borrowed_memory.mem()[addr..end.min(addr + 16)];

            // EDUCATIONAL: Convert bytes to hex strings
            let hex: Vec<String> = line.iter().map(|b| format!("{:02x}", b)).collect();
            let hex_str = hex.join(" ");

            // EDUCATIONAL: Convert bytes to ASCII (printable characters only)
            let ascii: String = line
                .iter()
                .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                .collect();

            println!("{:08x}  {:<47}  |{}|", addr, hex_str, ascii);
        }
        println!("-------------------");
    }

    /// Dumps the current state of all CPU registers for debugging.
    ///
    /// EDUCATIONAL PURPOSE: This demonstrates register inspection, which is
    /// essential for understanding program state and debugging issues.
    ///
    /// RISC-V REGISTER CONVENTIONS: The output shows both register numbers
    /// and their ABI (Application Binary Interface) names, which helps
    /// understand how registers are used in RISC-V programs.
    ///
    /// REGISTER USAGE:
    /// - x0 (zero): Always zero
    /// - x1 (ra): Return address
    /// - x2 (sp): Stack pointer
    /// - x10-x17 (a0-a7): Function arguments
    /// - x5-x7, x28-x31 (t0-t6): Temporary registers
    /// - x8, x9, x18-x27 (s0-s11): Saved registers
    pub fn dump_registers(&self) {
        println!("--- Register Dump ---");

        // EDUCATIONAL: RISC-V ABI register names for easier understanding
        const ABI_NAMES: [&str; 32] = [
            "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3",
            "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
            "t3", "t4", "t5", "t6",
        ];

        // EDUCATIONAL: Display each register with its name and value
        for i in 0..32 {
            let name = ABI_NAMES[i];
            let val = self.cpu.regs[i];
            println!("x{:02} ({:<4}) = 0x{:08x} ({})", i, name, val, val);
        }

        // EDUCATIONAL: Show program counter separately
        println!("pc           = 0x{:08x}", self.cpu.pc);
        println!("------------------------");
    }

    /// Starts program execution without initializing registers or setting up state.
    ///
    /// EDUCATIONAL PURPOSE: This is the main execution loop of the VM. It
    /// continuously fetches, decodes, and executes instructions until the
    /// program halts or encounters an error.
    ///
    /// EXECUTION LOOP: This implements the classic fetch-decode-execute cycle
    /// that all CPUs follow. The loop continues until the CPU returns false,
    /// indicating that execution should stop.
    ///
    /// ASSUMPTIONS: This function assumes the VM is already properly configured
    /// with code loaded and registers set up. For a complete VM, you'd typically
    /// call this after setting up the initial state.
    pub fn raw_run(&mut self) {
        // EDUCATIONAL: Main execution loop - fetch, decode, execute
        while self.cpu.step(
            Rc::clone(&self.memory),
            Rc::clone(&self.storage),
            &mut self.host,
        ) {}
    }
}
