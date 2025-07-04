mod tests;

use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::transaction::{TransactionType};
use types::address::Address;
use avm::avm::AVM;
use state::State;
use avm::global::Config;
use crate::tests::TEST_CASES;

pub const VM_MEMORY_SIZE: usize = 10 * 1024; // 10 KB
pub const MAX_MEMORY_PAGES: usize = 1; 

#[test]
fn test_entrypoint_function() {
    for case in TEST_CASES.iter() {
        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        for tx in transactions {
            match tx.tx_type {
                TransactionType::Transfer => {
                    panic!("transfer test case not implemented");
                }

                TransactionType::ProgramCall => {
                    println!("--- Program call test `{}` ---", case.name);
            
                    // 1. Run VM
                    let res = avm.run_tx(tx);
                    
                    // 2. Print result
                    avm.state.pretty_print();
                    println!("Success: {}, Error code: {}\n", res.success, res.error_code);

                    assert_eq!(res.success, case.expected_success, "Expected success = {} but got {}", case.expected_success, res.success);
                    assert_eq!(res.error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, res.error_code);
                }

                TransactionType::CreateAccount => {
                    println!("--- Account create test `{}` ---", case.name);
                    // 1. get code
                    let code = get_program_code(case.path);

                    // 2. set into tx
                    let mut tx_cpy = tx.clone();
                    tx_cpy.data = code;

                    // 3. run and get result
                    let res = avm.run_tx(tx_cpy);
                    avm.state.pretty_print();
                    println!("Success: {}, Error code: {}\n", res.success, res.error_code);
                }
            }
        }
    }
}

pub fn get_program_code<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let path_str = path.as_ref().display();
    let bytes = fs::read(&path)
        .unwrap_or_else(|_| panic!("❌ Failed to read ELF file from {}", path_str));

    let elf = parse_elf_from_bytes(&bytes)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ELF from {}", path_str));

    println!("✅ Parsed ELF: {} ({} sections)", path_str, elf.sections.len());

    let (code, code_start) = elf
    .get_flat_code()
    .unwrap_or_else(|| panic!("❌ No code sections found in ELF {}", path_str));

    let (rodata, rodata_start) = elf
        .get_flat_rodata()
        .unwrap_or_else(|| {
            println!("⚠️ No .rodata section found in ELF {}", path_str);
            (vec![], usize::MAX as u64)
        });

    // assert sizes
    assert!(code.len() <= Config::CODE_SIZE_LIMIT, "code size exceeds limit");
    assert!(rodata.len() <= Config::RO_DATA_SIZE_LIMIT, "read only data size exceeds limit");

    let mut total_len = code_start + code.len() as u64; // assumes rodata is after code
    if rodata.len() > 0 {
        total_len = rodata_start + rodata.len() as u64; // assumes rodata is after code
    }

    // Initialize memory with 0x00
    let mut combined = vec![0u8; total_len as usize];

    // Copy code
    combined[code_start as usize..code_start as usize + code.len()].copy_from_slice(&code);
    println!(
        "📦 Code size: {} bytes, starting at 0x{:08x}",
        code.len(),
        code_start
    );

    // Copy rodata (if it exists)
    if rodata.len() > 0 {
        combined[rodata_start as usize..rodata_start as usize + rodata.len()].copy_from_slice(&rodata);
        println!(
            "📦 Readonly data size: {} bytes, starting at 0x{:08x}",
            rodata.len(),
            rodata_start
        );
    }
    combined
}

pub fn populate_state<P: AsRef<Path>>(path: P, address: Address, state: &mut State) {
    let code = get_program_code(path);

    // deploy state
    state.deploy_contract(address, code);
}
