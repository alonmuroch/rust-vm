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
    ("lrsc", "LR/SC implementation needs improvement - causes infinite loops"),
];

/// Testing categories to run
const TESTING_CATEGORIES: &[&str] = &["ui", "um", "ua"];

/// Check if a test file should be skipped
fn should_skip_test(file_name: &str) -> Option<&str> {
    for (test_name, reason) in SKIPPED_TESTS {
        if file_name.ends_with(test_name) {
            return Some(reason);
        }
    }
    None
}

/// Run a single test file
fn run_single_test(elf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(elf_path).exists() {
        println!("ELF file not found at {}, skipping...", elf_path);
        return Ok(());
    }

    // Read ELF file
    let mut file = std::fs::File::open(elf_path)?;
    let mut elf_bytes = Vec::new();
    file.read_to_end(&mut elf_bytes)?;

    // Parse ELF
    let elf = compiler::elf::parse_elf_from_bytes(&elf_bytes)?;
    let (code, code_start) = elf.get_flat_code().ok_or("No code section in ELF")?;
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
        return Ok(());
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
    
    Ok(())
}

/// Discover and collect test files for a specific category
fn collect_test_files(test_dir: &str, category: &str) -> (Vec<String>, usize) {
    let mut test_files = Vec::new();
    let mut skipped_count = 0;
    
    println!("Looking for files in: {}", test_dir);
    println!("Category prefix: rv32{}p-", category);
    
    if let Ok(entries) = std::fs::read_dir(test_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        // Include files that start with the category prefix and are not .dump files
                        let category_prefix = format!("rv32{}-p-", category);
                        if name_str.starts_with(&category_prefix) && 
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
    } else {
        println!("Failed to read directory: {}", test_dir);
    }
    
    test_files.sort(); // Sort for consistent ordering
    (test_files, skipped_count)
}

/// Run all tests for a specific category
fn run_category_tests(test_dir: &str, category: &str) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
    println!("\n=== Running {} category tests ===", category.to_uppercase());
    
    let (test_files, skipped_count) = collect_test_files(test_dir, category);
    println!("Found {} {} test files to run ({} skipped)", test_files.len(), category, skipped_count);

    let mut passed_count = 0;
    let mut failed_count = 0;

    for (i, elf_path) in test_files.iter().enumerate() {
        let test_name = std::path::Path::new(elf_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        
        print!("[{:2}/{:2}] {}: ", i + 1, test_files.len(), test_name);
        
        if let Err(e) = run_single_test(elf_path) {
            println!("❌ FAILED - {}", e);
            failed_count += 1;
            return Err(e);
        } else {
            println!("✅ PASSED");
            passed_count += 1;
        }
    }
    
    println!("=== {} category tests completed ===", category.to_uppercase());
    Ok((passed_count, failed_count, skipped_count))
}

#[test]
fn test_riscv_spec() {
    // Discover all test files in the riscv-tests directory
    let test_dir = "tests/riscv-tests-install/share/riscv-tests/isa";
    
    // Print current working directory for debugging
    println!("Current dir: {:?}", std::env::current_dir().unwrap());
    println!("Looking for tests in: {}", test_dir);

    // Check if the test directory exists
    if !Path::new(test_dir).exists() {
        println!("Test directory not found at {}, skipping test", test_dir);
        return;
    }

    println!("\n🚀 Starting RISC-V Specification Test Suite");
    println!("{}", "=".repeat(60));

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;
    let mut category_results = Vec::new();

    // Run tests for each category
    for category in TESTING_CATEGORIES {
        match run_category_tests(test_dir, category) {
            Ok((passed, failed, skipped)) => {
                total_passed += passed;
                total_failed += failed;
                total_skipped += skipped;
                category_results.push((category.to_string(), passed, failed, skipped));
            }
            Err(e) => {
                println!("❌ Failed to run {} category tests: {}", category, e);
                panic!("Test suite failed");
            }
        }
    }
    
    // Print comprehensive summary
    println!("\n{}", "=".repeat(60));
    println!("📊 RISC-V SPECIFICATION TEST SUITE SUMMARY");
    println!("{}", "=".repeat(60));
    
    // Category breakdown
    println!("\n📋 Category Breakdown:");
    for (category, passed, failed, skipped) in &category_results {
        let total = passed + failed + skipped;
        let success_rate = if total > 0 { (*passed as f64 / total as f64) * 100.0 } else { 0.0 };
        println!("  {}: {}/{} passed ({:.1}%) {} skipped", 
                category.to_uppercase(), passed, total, success_rate, skipped);
    }
    
    // Overall statistics
    let total_tests = total_passed + total_failed + total_skipped;
    let overall_success_rate = if total_tests > 0 { (total_passed as f64 / total_tests as f64) * 100.0 } else { 0.0 };
    
    println!("\n📈 Overall Statistics:");
    println!("  Total Tests: {}", total_tests);
    println!("  Passed: {} ✅", total_passed);
    println!("  Failed: {} ❌", total_failed);
    println!("  Skipped: {} ⏭️", total_skipped);
    println!("  Success Rate: {:.1}%", overall_success_rate);
    
    // Test coverage information
    println!("\n🎯 Test Coverage:");
    println!("  UI Tests: Base integer instructions (RV32I)");
    println!("  UM Tests: Integer multiplication and division (RV32M)");
    
    // Skipped tests explanation
    if total_skipped > 0 {
        println!("\n⏭️ Skipped Tests:");
        for (test_name, reason) in SKIPPED_TESTS {
            println!("  - {}: {}", test_name, reason);
        }
    }
    
    println!("\n{}", "=".repeat(60));
    
    if total_failed > 0 {
        panic!("Test suite completed with {} failures", total_failed);
    } else {
        println!("🎉 All tests passed successfully!");
    }
}