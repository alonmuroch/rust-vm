use core::convert::TryInto;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::transaction::Transaction;
use types::address::Address;
use types::result::Result;
use avm::avm::AVM;

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
                to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                data: vec![], // input data
                value: 0,
                nonce: 0,
            },
        },
        // TestCase {
        //     name: "simple",
        //     path: "../examples/bin/simple.elf",
        //     expected_success: true,
        //     expected_error_code: 100,
        //     transaction: Transaction {
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
        println!("--- Running entrypoint for `{}` ---", case.name);

        // 1. Create VM and load ELF using compiler crate
        // let context: ExecutionContext = ExecutionContext::new(case.transaction.to, case.transaction.from);
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        load_and_run_elf(case.path, &mut avm);
        // vm.cpu.verbose = true;
 
        // 2. Run VM
        let res = avm.run_tx(case.transaction);
        
        // 3. Print result
        println!("Success: {}, Error code: {}\n", res.success, res.error_code);

        assert_eq!(res.success, case.expected_success, "Expected success = {} but got {}", case.expected_success, res.success);
        assert_eq!(res.error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, res.error_code);
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

    bytes
}

pub fn load_and_run_elf<P: AsRef<Path>>(path: P, vm: &mut AVM) {
    let path_str = path.as_ref().display();
    let bytes = fs::read(&path)
        .unwrap_or_else(|_| panic!("❌ Failed to read ELF file from {}", path_str));

    let elf = parse_elf_from_bytes(&bytes)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ELF from {}", path_str));

    println!("✅ Parsed ELF: {} ({} sections)", path_str, elf.sections.len());

    let (code, code_start) = elf.get_flat_code()
        .unwrap_or_else(|| panic!("❌ No code sections found in ELF {}", path_str));
    println!("📦 Code size: {} bytes, starting at 0x{:08x}", code.len(), code_start);
    vm.set_code(code_start as usize, &code);

    if let Some((rodata, rodata_start)) = elf.get_flat_rodata() {
        println!(
            "📦 Readonly data size: {} bytes, starting at 0x{:08x}",
            rodata.len(),
            rodata_start
        );
        vm.set_rodata(rodata_start as usize, &rodata);
    } else {
        println!("⚠️ No .rodata section found in ELF {}", path_str);
    }
}
