use core::arch::asm;
use program::{log, logf};

use crate::syscall;

const SCAUSE_ECALL_FROM_U: usize = 8;
const SAVED_REG_COUNT: usize = 11;

/// Install the kernel trap vector and set up the kernel stack for traps.
pub fn init_trap_vector(kstack_top: u32) {
    logf!("init_trap_vector: kstack_top=0x%x", kstack_top);
    unsafe {
        asm!("csrw sscratch, {0}", in(reg) kstack_top);
        asm!("csrw stvec, {0}", in(reg) trap_entry as usize);
    }
}

/// Trap entry stub:
/// - Switch to the kernel stack via sscratch.
/// - Save sepc, ra, a0-a7, and t0.
/// - Call into the Rust trap handler with a pointer to the saved area.
/// - Restore registers and return with sret.
// #[unsafe(naked)]
pub unsafe extern "C" fn trap_entry() -> ! {
    unsafe {
        asm!(
            // Switch to kernel stack and make room for saved registers.
            "csrrw sp, sscratch, sp",
            "addi sp, sp, -44",
            // Save caller-saved registers we clobber and sepc.
            "sw t0, 40(sp)",
            "csrr t0, sepc",
            "sw t0, 0(sp)",   // saved sepc
            "sw ra, 4(sp)",
            "sw a0, 8(sp)",
            "sw a1, 12(sp)",
            "sw a2, 16(sp)",
            "sw a3, 20(sp)",
            "sw a4, 24(sp)",
            "sw a5, 28(sp)",
            "sw a6, 32(sp)",
            "sw a7, 36(sp)",
            // Call Rust trap handler with pointer to the save area in a0.
            "mv a0, sp",
            "call {handler}",
            // Restore sepc and registers, then return from trap.
            "lw t0, 0(sp)",
            "csrw sepc, t0",
            "lw ra, 4(sp)",
            "lw a0, 8(sp)",
            "lw a1, 12(sp)",
            "lw a2, 16(sp)",
            "lw a3, 20(sp)",
            "lw a4, 24(sp)",
            "lw a5, 28(sp)",
            "lw a6, 32(sp)",
            "lw a7, 36(sp)",
            "lw t0, 40(sp)",
            "addi sp, sp, 44",
            "csrrw sp, sscratch, sp",
            "sret",
            handler = sym handle_trap
        );
        core::hint::unreachable_unchecked();
    }
}

/// Rust-level trap handler. Receives a pointer to the saved register block
/// laid out as:
/// [0] sepc, [1] ra, [2] a0, [3] a1, [4] a2, [5] a3, [6] a4, [7] a5,
/// [8] a6, [9] a7, [10] t0.
#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(saved: *mut u32) {
    let regs = unsafe { core::slice::from_raw_parts_mut(saved, SAVED_REG_COUNT) };
    let scause = read_scause();
    let stval = read_stval();
    let sepc = regs[0];
    let satp = read_satp();
    let sp: u32;
    unsafe { asm!("mv {0}, sp", out(reg) sp); }
    
    // logf!(
    //     "trap_entry: scause=0x%x stval=0x%x sepc=0x%x satp=0x%x sp=0x%x",
    //     scause as u32,
    //     stval as u32,
    //     sepc,
    //     satp,
    //     sp
    // );
    let is_interrupt = (scause >> 31) != 0;
    if is_interrupt {
        panic!(
            "unexpected interrupt trap: scause=0x{:x} stval=0x{:x} sepc=0x{:08x}",
            scause, stval, sepc
        );
    }

    let code = scause & 0xfff;
    match code {
        SCAUSE_ECALL_FROM_U => {
            let args = [
                regs[3], // a1
                regs[4], // a2
                regs[5], // a3
                regs[6], // a4
                regs[7], // a5
                regs[8], // a6
            ];
            let call_id = regs[9]; // a7
            let ret = syscall::dispatch_syscall(call_id, args);
            regs[2] = ret; // a0 return value
            regs[0] = regs[0].wrapping_add(4); // Advance past ecall
        }
        _ => log!("unhandled trap"),
    }
}

#[inline(always)]
fn read_scause() -> usize {
    let value: usize;
    unsafe { asm!("csrr {0}, scause", out(reg) value); }
    value
}

#[inline(always)]
fn read_satp() -> u32 {
    let value: u32;
    unsafe { asm!("csrr {0}, satp", out(reg) value); }
    value
}

#[inline(always)]
fn read_stval() -> usize {
    let value: usize;
    unsafe { asm!("csrr {0}, stval", out(reg) value); }
    value
}
