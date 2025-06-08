use std::fs;
use vm::cpu::CPU;
use vm::registries::Register;

struct TestCase<'a> {
    name: &'a str,
    path: &'a str,
    expected_x10: u32,
    input_regs: &'a [(Register, u32)], // (register index, value)
}

#[test]
fn test_multiple_riscv_programs() {
    let test_cases = [
        TestCase {
            name: "simple",
            path: "tests/programs/bin/simple.bin",
            expected_x10: 15,
            input_regs: &[], // no inputs needed
        },
        TestCase {
            name: "simple flow control",
            path: "tests/programs/bin/simple_flow_control.bin",
            expected_x10: 1,
            input_regs: &[(Register::A0, 7), (Register::A1, 3)],
        },
        TestCase {
            name: "simple flow control #2",
            path: "tests/programs/bin/simple_flow_control.bin",
            expected_x10: 2,
            input_regs: &[(Register::A0, 3), (Register::A1, 7)],
        },
    ];

    for case in test_cases {
        println!("--- Running program: {} ---", case.name);

        let binary = fs::read(case.path).unwrap_or_else(|_| panic!("Failed to load {}", case.path));
        let mut cpu = CPU::new(binary);
        cpu.verbose = true;

        // Set input registers
        for &(reg, val) in case.input_regs {
            cpu.regs[reg as usize] = val;
        }

        let result = cpu.run_with_trace(100);

        println!("Trace:\n{}", result);
        println!("Final x10: {}\n", cpu.regs[10]);

        assert_eq!(
            cpu.regs[10], case.expected_x10,
            "Program `{}` failed: expected x10 = {}, got {}",
            case.name, case.expected_x10, cpu.regs[10]
        );
    }
}
