use vm::vm::VM;
use vm::registers::Register;
use core::convert::TryInto;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const VM_MEMORY_SIZE: usize = 10 * 1024; // 10 KB

#[derive(Debug)]
struct TestCase<'a> {
    name: &'a str,
    path: &'a str,
    expected_success: bool,
    expected_error_code: u32,
    pubkey: [u8; 32],
    input: &'a [u8],
}

#[derive(Debug)]
struct ResultStruct {
    pub success: bool,
    pub error_code: u32,
}

#[test]
fn test_entrypoint_function() {
    let test_cases = [
        // TestCase {
        //     name: "storage",
        //     path: "../examples/bin/storage.elf",
        //     expected_success: true,
        //     expected_error_code: 5,
        //     pubkey: from_hex("e4a3c7f85d2b6e91fa78cd3210b45f6ae913d07c2ba9961e4f5c88a2de3091bc"),
        //     input: &[],
        // },
        TestCase {
            name: "simple",
            path: "../examples/bin/simple.elf",
            expected_success: true,
            expected_error_code: 100,
            pubkey: from_hex("e4a3c7f85d2b6e91fa78cd3210b45f6ae913d07c2ba9961e4f5c88a2de3091bc"),
            input: &[
                100, 0, 0, 0,   // first u64 = 100
                42, 0, 0, 0,      // second u64 = 42
            ],
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
        let result = catch_unwind(AssertUnwindSafe(|| {
            vm.run(); // this might panic
        }));
        // Always dump memory and storage
        vm.dump_memory(0, VM_MEMORY_SIZE);
        vm.dump_storage();
        vm.dump_registers();
        if let Err(e) = result {
            eprintln!("💥 VM panicked: {:?}", e);
            panic!("VM panicked");
        }
        
        // 4. Extract result struct
        let res = extract_and_print_result(&vm, result_ptr);
        println!("Success: {}, Error code: {}\n", res.success, res.error_code);

        assert_eq!(res.success, case.expected_success, "Expected success = {} but got {}", case.expected_success, res.success);
        assert_eq!(res.error_code, case.expected_error_code, "Expected error_code = {} but got {}", case.expected_error_code, res.error_code);
    }
}

fn extract_and_print_result(vm: &VM, result_ptr: u32) -> ResultStruct {
    let mem = vm.memory.mem();
    let start = result_ptr as usize;

    if start + 5 > mem.len() {
        panic!("Result struct out of bounds at 0x{:08x}", start);
    }

    let error_code = u32::from_le_bytes(mem[start..start + 4].try_into().unwrap());
    let success = mem[start + 4] != 0;

    ResultStruct { error_code, success }
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
    for i in 0..32 {
        let hi = from_hex_char(hex_bytes[i * 2]);
        let lo = from_hex_char(hex_bytes[i * 2 + 1]);
        bytes[i] = (hi << 4) | lo;
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
