use std::fs;
use vm::cpu::CPU;

#[test]
fn test_simple_riscv_program() {
    // Load the compiled binary
    let binary = fs::read("tests/programs/bin/simple.bin")
        .expect("Failed to load simple.bin");

    // Run the program with tracing
    let mut cpu = CPU::new(binary);
    let result = cpu.run_with_trace(50); // max 50 steps

    // Output the register dump
    println!("{}", result);

    // Expect result in x10 (a0)
    assert_eq!(cpu.regs[10], 15, "Expected x10 to be 15 (5 + 10)");
}
