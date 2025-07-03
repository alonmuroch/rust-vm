use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::transaction::{TransactionType, Transaction};
use types::address::Address;
use avm::avm::AVM;
use state::State;
use avm::global::Config;

pub const VM_MEMORY_SIZE: usize = 10 * 1024; // 10 KB
pub const MAX_MEMORY_PAGES: usize = 1; 

#[derive(Debug)]
struct TestCase<'a> {
    name: &'a str,
    path: &'a str,
    expected_success: bool,
    expected_error_code: u32,
    transaction: Transaction,
}

#[test]
fn test_entrypoint_function() {
    let test_cases = [
        TestCase {
            name: "storage",
            path: "../examples/bin/storage.elf",
            expected_success: true,
            expected_error_code: 0,
            transaction: Transaction {
                tx_type: TransactionType::CreateAccount,
                to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                data: vec![], // input data
                value: 0,
                nonce: 0,
            },
        },
        // TestCase {
        //     name: "storage",
        //     path: "../examples/bin/storage.elf",
        //     expected_success: true,
        //     expected_error_code: 0,
        //     transaction: Transaction {
        //         tx_type: TransactionType::ProgramCall,
        //         to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //         from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //         data: vec![], // input data
        //         value: 0,
        //         nonce: 0,
        //     },
        // },
        // TestCase {
        //     name: "simple",
        //     path: "../examples/bin/simple.elf",
        //     expected_success: true,
        //     expected_error_code: 100,
        //     transaction: Transaction {
        //         tx_type: TransactionType::ProgramCall,
        //         to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //         from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //         data: vec![
        //             100, 0, 0, 0,   // first u64 = 100
        //             42, 0, 0, 0,      // second u64 = 42
        //         ], // input data
        //         value: 0,
        //         nonce: 0,
        //     },
        // },
    ];

    for case in test_cases {
        match case.transaction.tx_type {
            TransactionType::Transfer => {
                panic!("transfer test case not implemented");
            }

            TransactionType::ProgramCall => {
                println!("--- Program call test `{}` ---", case.name);

                // 1. Create VM and load ELF using compiler crate
                let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
                populate_state(case.path, case.transaction.to, &mut avm.state);
                // vm.cpu.verbose = true;
        
                // 2. Run VM
                let res = avm.run_tx(case.transaction);
                
                // 3. Print result
                avm.state.pretty_print();
                println!("Success: {}, Error code: {}\n", res.success, res.error_code);

                assert_eq!(res.success, case.expected_success, "Expected success = {} but got {}", case.expected_success, res.success);
                assert_eq!(res.error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, res.error_code);
            }

            TransactionType::CreateAccount => {
                // 1. Create VM and load ELF using compiler crate
                let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);

                // 2. get code
                let code = get_program_code(case.path);

                // 3. set into tx
                let mut tx = case.transaction.clone();
                tx.data = code;

                let res = avm.run_tx(tx);
                avm.state.pretty_print();
                println!("Success: {}, Error code: {}\n", res.success, res.error_code);
            }
        }

        
    }
}

pub fn to_address(hex: &str) -> Address {
    assert!(hex.len() == 40, "Hex string must be exactly 40 characters");

    fn from_hex_char(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("Invalid hex character"),
        }
    }

    let mut bytes = [0u8; 20];
    let hex_bytes = hex.as_bytes();
    for i in 0..20 {
        let hi = from_hex_char(hex_bytes[i * 2]);
        let lo = from_hex_char(hex_bytes[i * 2 + 1]);
        bytes[i] = (hi << 4) | lo;
    }

    Address::new(bytes)
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

    let total_len = rodata_start + rodata.len() as u64; // assumes rodata is after code

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
    if rodata_start > 0 {
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
