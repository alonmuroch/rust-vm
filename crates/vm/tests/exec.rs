use vm::cpu::CPU; // assuming CPU is pub in src/cpu.rs
// use vm::decoder::*; // if needed

#[test]
fn test_addi_program() {
    // RISC-V machine code:
    // addi x1, x0, 5   --> 0x00500093
    // addi x2, x1, 10  --> 0x00a08113
    let code: Vec<u8> = vec![
        0x93, 0x00, 0x50, 0x00, // addi x1, x0, 5
        0x13, 0x81, 0xa0, 0x00, // addi x2, x1, 10
    ];

    let mut cpu = CPU::new(code);
    assert!(cpu.step());
    assert_eq!(cpu.regs[1], 5);

    assert!(cpu.step());
    assert_eq!(cpu.regs[2], 15);
}
