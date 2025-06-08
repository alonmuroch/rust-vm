use std::fs;
use vm::cpu::CPU;
use vm::vm::VM;
use vm::registries::Register;

struct TestCase<'a> {
    name: &'a str,
    path: &'a str,
    function: &'a str, // name of the exported function to call
    expected_x10: u32,
    input_regs: &'a [(Register, u32)],
}

#[test]
fn test_multiple_riscv_functions() {
    let test_cases = [
        TestCase {
            name: "simple",
            path: "tests/programs/bin/simple.o",
            function: "main",
            expected_x10: 15,
            input_regs: &[],
        },
        // TestCase {
        //     name: "simple flow control",
        //     path: "tests/programs/bin/simple_flow_control.o",
        //     function: "main",
        //     expected_x10: 1,
        //     input_regs: &[(Register::A0, 7), (Register::A1, 3)],
        // },
        // TestCase {
        //     name: "simple flow control #2",
        //     path: "tests/programs/bin/simple_flow_control.o",
        //     function: "main",
        //     expected_x10: 2,
        //     input_regs: &[(Register::A0, 3), (Register::A1, 7)],
        // },
    ];

    for case in test_cases {
        println!("--- Running function: {} in `{}` ---", case.function, case.name);

        let code = fs::read(case.path).unwrap_or_else(|_| panic!("Failed to load {}", case.path));
        let mut vm = VM::new(code);
        vm.cpu.verbose = true;

        // Set input registers before the function call
        for &(reg, val) in case.input_regs {
            vm.cpu.regs[reg as usize] = val;
        }

        vm.call(case.function);

        println!("Final x10: {}\n", vm.cpu.regs[10]);

        assert_eq!(
            vm.cpu.regs[10],
            case.expected_x10,
            "Function `{}` in program `{}` failed: expected x10 = {}, got {}",
            case.function,
            case.name,
            case.expected_x10,
            vm.cpu.regs[10]
        );
    }
}
