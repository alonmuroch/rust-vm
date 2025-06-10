use std::fs;
use vm::vm::VM;
use vm::registers::Register;
use core::convert::TryInto;

pub const VM_MEMORY_SIZE: usize = 5 * 1024; // 10 KB

#[derive(Debug)]
struct TestCase<'a> {
    name: &'a str,
    path: &'a str,
    expected_success: bool,
    expected_error_code: u32,
    pubkey: [u8; 32],
    input: &'a [u8],
}

#[test]
fn test_entrypoint_function() {
    let test_cases = [
        TestCase {
            name: "compare_two_numbers",
            path: "tests/programs/bin/multi_func.bin",
            expected_success: true,
            expected_error_code: 1,
            pubkey: from_hex("e4a3c7f85d2b6e91fa78cd3210b45f6ae913d07c2ba9961e4f5c88a2de3091bc"),
            input: &[
                100, 0, 0, 0, 0, 0, 0, 0,   // first u64 = 100
                42, 0, 0, 0, 0, 0, 0, 0     // second u64 = 42
            ],
        },
        TestCase {
            name: "compare_two_numbers #2",
            path: "tests/programs/bin/multi_func.bin",
            expected_success: true,
            expected_error_code: 2,
            pubkey: from_hex("e4a3c7f85d2b6e91fa78cd3210b45f6ae913d07c2ba9961e4f5c88a2de3091bc"),
            input: &[
                25, 0, 0, 0, 0, 0, 0, 0,   // first u64 = 100
                50, 0, 0, 0, 0, 0, 0, 0     // second u64 = 42
            ],
        }
    ];

    for case in test_cases {
        println!("--- Running entrypoint for `{}` ---", case.name);

        // 1. Create VM and load code
        let code = fs::read(case.path)
            .unwrap_or_else(|_| panic!("Failed to load {}", case.path));
        let mut vm = VM::new(VM_MEMORY_SIZE); // 64 KB memory
        vm.cpu.verbose = true;
        vm.set_code(&code);

        // 2. Allocate memory and set registers
        let pubkey_ptr = vm.set_reg_to_data(Register::A0, &case.pubkey);
        let input_ptr = vm.set_reg_to_data(Register::A1, case.input);
        vm.set_reg_to_data(Register::A2, &case.input.len().to_le_bytes());
        let result_ptr = vm.set_reg_to_data(Register::A3, &[0u8; 5]);

        // vm.dump_memory(0, VM_MEMORY_SIZE);

        // 3. Execute program
        vm.run();
        vm.dump_memory(0, VM_MEMORY_SIZE);

        // vm.dump_memory(0, VM_MEMORY_SIZE);

        // 4. Read result struct from memory
        let mem = vm.memory.mem(); // Borrow memory
        let start = result_ptr as usize;
        if start + 5 > mem.len() {
            panic!("Result struct out of bounds at 0x{:08x}", start);
        }
        let error_code = u32::from_le_bytes(mem[start..start + 4].try_into().unwrap());
        let success = mem[start + 4] != 0;

        println!("Success: {}, Error code: {}\n", success, error_code);

        assert_eq!(success, case.expected_success, "Expected success = {} but got {}", case.expected_success, success);
        assert_eq!(error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, error_code);
    }
}

pub fn from_hex(hex: &str) -> [u8; 32] {
    assert!(hex.len() == 64, "Hex string must be exactly 64 characters");

    fn from_hex_char(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("Invalid hex character"),
        }
    }

    let mut bytes = [0u8; 32];
    let hex_bytes = hex.as_bytes();
    let mut i = 0;
    while i < 32 {
        let hi = from_hex_char(hex_bytes[i * 2]);
        let lo = from_hex_char(hex_bytes[i * 2 + 1]);
        bytes[i] = (hi << 4) | lo;
        i += 1;
    }

    bytes
}