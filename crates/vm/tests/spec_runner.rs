//! Standalone test runner for rv32ui-p-* ELF files from riscv-tests
//! Loads an ELF file, loads it into the VM, and runs it to completion.

use std::io::Read;
use vm::vm::VM;
mod test_syscall_handler;
use test_syscall_handler::TestSyscallHandler;

#[test]
fn test_riscv_spec() {
    // Path to a single rv32ui ELF file (change as needed)
    let elf_path = "tests/riscv-tests-install/share/riscv-tests/isa/rv32ui-p-add";
    println!("Loading test: {}", elf_path);

    // Print current working directory for debugging
    println!("Current dir: {:?}", std::env::current_dir().unwrap());
    if !std::path::Path::new(elf_path).exists() {
        panic!("ELF file not found at {}", elf_path);
    }
    // Read ELF file
    let mut file = std::fs::File::open(elf_path).expect("Failed to open ELF file");
    let mut elf_bytes = Vec::new();
    file.read_to_end(&mut elf_bytes).expect("Failed to read ELF file");

    // Parse ELF
    let elf = compiler::elf::parse_elf_from_bytes(&elf_bytes).expect("Failed to parse ELF");
    let (code, code_start) = elf.get_flat_code().expect("No code section in ELF");
    let (rodata, rodata_start) = elf.get_flat_rodata().unwrap_or((vec![], usize::MAX as u64));

    // Set up VM memory (allocate enough to cover 0x80000000+)
    let memory = std::rc::Rc::new(std::cell::RefCell::new(vm::memory_page::MemoryPage::new_with_base(0x20000, 0x80000000))); // 128KB at 0x80000000
    // Only use vm.set_code to load code
    println!("Loading code into VM: addr=0x{:x}, size=0x{:x}", code_start, code.len());

    // Set up VM
    let storage = std::rc::Rc::new(std::cell::RefCell::new(storage::Storage::default()));
    let host: Box<dyn vm::host_interface::HostInterface> = Box::new(vm::host_interface::NoopHost {});
    // When constructing the VM, use the test syscall handler:
    let mut vm = VM::new_with_syscall_handler(
        memory.clone(),
        storage,
        host,
        Box::new(TestSyscallHandler::new()),
    );
    vm.cpu.verbose = true;
    vm.set_code(code_start as u32, code_start as u32, &code);

    if !rodata.is_empty() {
        println!("Writing rodata to memory: addr=0x{:x}, size=0x{:x}", rodata_start, rodata.len());
        memory.borrow_mut().write_code(rodata_start as usize, &rodata);
    }

    // Run the VM
    println!("Running test...");
    vm.raw_run();
    println!("Test completed.");
    // vm.dump_all_memory();

    // TODO: Check result (registers, memory, etc.)
}
