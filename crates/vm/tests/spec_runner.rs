//! Standalone test runner for rv32ui-p-* ELF files from riscv-tests
//! Loads an ELF file, loads it into the VM, and runs it to completion.

use std::io::Read;
use std::path::Path;
use vm::vm::VM;
mod test_syscall_handler;
use test_syscall_handler::TestSyscallHandler;

/// Tests that are skipped and the reasons why
const SKIPPED_TESTS: &[(&str, &str)] = &[
    ("fence_i", "Requires self-modifying code support (writes instructions to memory and executes them)"),
    ("ld_st", "Contains 64-bit load/store instructions (ld/sd) that the 32-bit VM doesn't support"),
    ("st_ld", "Contains 64-bit store/load instructions (sd/ld) that the 32-bit VM doesn't support"),
];

/// Check if a test file should be skipped
fn should_skip_test(file_name: &str) -> Option<&str> {
    for (test_name, reason) in SKIPPED_TESTS {
        if file_name.ends_with(test_name) {
            return Some(reason);
        }
    }
    None
}

#[test]
fn test_riscv_spec() {
    // Discover all test files in the riscv-tests directory
    let test_dir = "tests/riscv-tests/isa";
    
    // Print current working directory for debugging
    println!("Current dir: {:?}", std::env::current_dir().unwrap());
    println!("Looking for tests in: {}", test_dir);

    // Check if the test directory exists
    if !Path::new(test_dir).exists() {
        println!("Test directory not found at {}, skipping test", test_dir);
        return;
    }

    // Collect all rv32ui-p-* and rv32um-p-* files
    let mut test_files = Vec::new();
    let mut skipped_count = 0;
    if let Ok(entries) = std::fs::read_dir(test_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        // Include files that start with rv32ui-p- or rv32um-p- and are not .dump files
                        if (name_str.starts_with("rv32ui-p-") || name_str.starts_with("rv32um-p-")) && 
                           !path.is_dir() && 
                           !name_str.ends_with(".dump") {
                            
                            // Check if this test should be skipped
                            if let Some(reason) = should_skip_test(name_str) {
                                println!("Skipping {}: {}", name_str, reason);
                                skipped_count += 1;
                                continue;
                            }
                            
                            test_files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    test_files.sort(); // Sort for consistent ordering
    println!("Found {} test files to run ({} skipped)", test_files.len(), skipped_count);

    for (i, elf_path) in test_files.iter().enumerate() {
        println!("\n=== Running test {}: {} ===", i + 1, elf_path);
        
        if !Path::new(elf_path).exists() {
            println!("ELF file not found at {}, skipping...", elf_path);
            continue;
        }

        // Read ELF file
        let mut file = std::fs::File::open(elf_path).expect("Failed to open ELF file");
        let mut elf_bytes = Vec::new();
        file.read_to_end(&mut elf_bytes).expect("Failed to read ELF file");

        // Parse ELF
        let elf = compiler::elf::parse_elf_from_bytes(&elf_bytes).expect("Failed to parse ELF");
        let (code, code_start) = elf.get_flat_code().expect("No code section in ELF");
        let (rodata, rodata_start) = elf.get_flat_rodata().unwrap_or((vec![], usize::MAX as u64));
        
        // Get .data section if it exists
        let (data, data_start) = if let Some(data_section) = elf.get_section_by_name(".data") {
            (data_section.data.to_vec(), data_section.addr as usize)
        } else {
            (vec![], usize::MAX)
        };

        // Find .tohost section
        if let Some(tohost_section) = elf.get_section_by_name(".tohost") {
            println!(".tohost section found at addr=0x{:x}, size=0x{:x}", tohost_section.addr, tohost_section.size);
        } else {
            println!(".tohost section not found, skipping...");
            continue;
        }

        // Set up VM memory (allocate enough to cover 0x80000000+)
        let memory = std::rc::Rc::new(std::cell::RefCell::new(vm::memory_page::MemoryPage::new_with_base(0x20000, 0x80000000))); // 128KB at 0x80000000
        println!("Loading code into VM: addr=0x{:x}, size=0x{:x}", code_start, code.len());

        // Set up VM
        let storage = std::rc::Rc::new(std::cell::RefCell::new(storage::Storage::default()));
        let host: Box<dyn vm::host_interface::HostInterface> = Box::new(vm::host_interface::NoopHost {});
        // When constructing the VM, use the test syscall handler:
        let mut syscall_handler = Box::new(TestSyscallHandler::new());
        
        // Set .tohost address if found
        if let Some(tohost_section) = elf.get_section_by_name(".tohost") {
            syscall_handler.set_tohost_addr(tohost_section.addr);
            syscall_handler.set_memory(memory.clone());
        }
        
        // Move the handler into the VM, then extract it after run
        let mut vm = VM::new_with_syscall_handler(
            memory.clone(),
            storage,
            host,
            syscall_handler,
        );
        vm.cpu.verbose = false; // Set to false to reduce output for multiple tests
        vm.set_code(code_start as u32, code_start as u32, &code);

        if !rodata.is_empty() {
            println!("Writing rodata to memory: addr=0x{:x}, size=0x{:x}", rodata_start, rodata.len());
            memory.borrow_mut().write_code(rodata_start as usize, &rodata);
        }

        if !data.is_empty() {
            println!("Writing data to memory: addr=0x{:x}, size=0x{:x}", data_start, data.len());
            memory.borrow_mut().write_code(data_start as usize, &data);
        }

        // Run the VM
        println!("Running test...");
        vm.raw_run();
        println!("Test completed.");
    }
    
    println!("\n=== All tests completed ===");
}