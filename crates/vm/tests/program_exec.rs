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
            expected_error_code: 0,
            pubkey: [0u8; 32],
            input: &[
                100, 0, 0, 0, 0, 0, 0, 0,   // first u64 = 100
                42, 0, 0, 0, 0, 0, 0, 0     // second u64 = 42
            ],
        },
        TestCase {
            name: "equal_numbers",
            path: "tests/programs/bin/multi_func.bin",
            expected_success: false,
            expected_error_code: 0,
            pubkey: [1u8; 32],
            input: &[
                77, 0, 0, 0, 0, 0, 0, 0,    // first u64 = 77
                77, 0, 0, 0, 0, 0, 0, 0     // second u64 = 77
            ],
        },
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
        let result_ptr = vm.set_reg_to_data(Register::A3, &[0u8; 5]);

        vm.dump_memory(0, VM_MEMORY_SIZE);

        // 3. Execute program
        vm.run();

        // vm.dump_memory(0, VM_MEMORY_SIZE);

        // 4. Read result struct from memory
        let result = &vm.cpu.memory[result_ptr as usize..result_ptr as usize + 8];
        let error_code = u32::from_le_bytes(result[0..4].try_into().unwrap());
        let success = result[4] != 0;

        println!("Success: {}, Error code: {}\n", success, error_code);

        assert_eq!(success, case.expected_success, "Expected success = {} but got {}", case.expected_success, success);
        assert_eq!(error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, error_code);
    }
}
