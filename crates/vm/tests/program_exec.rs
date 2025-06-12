use vm::vm::VM;
use vm::registers::Register;
use core::convert::TryInto;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;

pub const VM_MEMORY_SIZE: usize = 5 * 1024; // 5 KB

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
            name: "storage",
            path: "tests/programs/bin/storage.elf",
            expected_success: true,
            expected_error_code: 1,
            pubkey: from_hex("e4a3c7f85d2b6e91fa78cd3210b45f6ae913d07c2ba9961e4f5c88a2de3091bc"),
            input: &[],
        },
    ];

    for case in test_cases {
        println!("--- Running entrypoint for `{}` ---", case.name);

        // 1. Create VM and load ELF using compiler crate
        let mut vm = VM::new(VM_MEMORY_SIZE);
        load_and_run_elf(case.path, &mut vm);
        vm.cpu.verbose = true;

        // 2. Inject input registers
        let _pubkey_ptr = vm.set_reg_to_data(Register::A0, &case.pubkey);
        let _input_ptr = vm.set_reg_to_data(Register::A1, case.input);
        vm.set_reg_u32(Register::A2, case.input.len() as u32);
        let result_ptr = vm.set_reg_to_data(Register::A3, &[0u8; 5]);

        // 3. Run VM
        vm.run();
        vm.dump_memory(0, VM_MEMORY_SIZE);
        
        // 4. Extract result struct
        let mem = vm.memory.mem();
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

pub fn load_and_run_elf<P: AsRef<Path>>(path: P, vm: &mut VM) {
    let path_str = path.as_ref().display();
    let bytes = fs::read(&path)
        .unwrap_or_else(|_| panic!("❌ Failed to read ELF file from {}", path_str));

    let elf = parse_elf_from_bytes(&bytes)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ELF from {}", path_str));

    println!("✅ Parsed ELF: {} ({} sections)", path_str, elf.sections.len());

    for section in &elf.sections {
        let vma = section.addr as usize;
        let file_offset = section.addr as usize;
        let size = section.size as usize;

        if size == 0 {
            continue; // Skip empty sections
        }

        match section.name.as_str() {
            ".text.entrypoint" => {
                println!("📦 Loading code section: {} at 0x{:08x} ({} bytes)", section.name, vma, size);
                let data = &bytes[file_offset..file_offset + size];
                vm.set_code(vma, data);
            }
            ".rodata" => {
                println!("📦 Loading rodata section: {} at 0x{:08x} ({} bytes)", section.name, vma, size);
                let data = &bytes[file_offset..file_offset + size];
                vm.set_rodata(vma, data);
            }
            _ => {}
        }
    }
}

