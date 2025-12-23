#![allow(dead_code)]

use crate::{AddressSpace, Config, Task, mmu};
use crate::global::NEXT_ASID;
use program::log;
use program::logf;
use types::{address::Address, ADDRESS_LEN};

const PAGE_SIZE: usize = 4096;
const STACK_BYTES: usize = 0x4000; // 16 KiB user stack
const HEAP_BYTES: usize = 0x8000; // 32 KiB user heap
pub const PROGRAM_VA_BASE: u32 = 0x0;
const fn align_up(val: usize, align: usize) -> usize {
    (val + (align - 1)) & !(align - 1)
}

/// Total mapped window for a program: code/rodata, stack, and heap.
pub const PROGRAM_WINDOW_BYTES: usize = align_up(
    Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT + STACK_BYTES + HEAP_BYTES,
    PAGE_SIZE,
);

const REG_SP: usize = 2;
const REG_A0: usize = 10;
const REG_A1: usize = 11;
const REG_A2: usize = 12;
const REG_A3: usize = 13;

const TO_PTR_ADDR: u32 = 0x120;
const FROM_PTR_ADDR: u32 = TO_PTR_ADDR + ADDRESS_LEN as u32;
const INPUT_BASE_ADDR: u32 = Config::HEAP_START_ADDR as u32;

/// Create a new task for a program and map its virtual address window via syscalls.
///
/// This sets up:
/// - Maps a fixed VA window [PROGRAM_VA_BASE, PROGRAM_VA_BASE + PROGRAM_WINDOW_BYTES).
/// - Returns a Task with the new address space and provided kernel stack top.
///
/// The caller is responsible for copying program bytes into the mapped window
/// and initializing the user trapframe (PC/SP/args) before running.
pub fn launch_program(
    kstack_top: u32,
    to: &Address,
    from: &Address,
    code: &[u8],
    input: &[u8],
) -> Option<Task> {
    if input.len() > Config::MAX_INPUT_LEN {
        log!("launch_program: input too large");
        return None;
    }

    let asid = alloc_asid();
    let root_ppn = match mmu::alloc_root() {
        Some(ppn) => ppn,
        None => {
            logf!("launch_program: no free root PPN available");
            return None;
        }
    };
    let window_end = PROGRAM_VA_BASE.wrapping_add(PROGRAM_WINDOW_BYTES as u32);
    logf!(
        "launch_program: asid=%d root=0x%x map=[0x%x,0x%x)",
        asid as u32,
        root_ppn,
        PROGRAM_VA_BASE,
        window_end
    );
    let perms = mmu::PagePerms::user_rwx();
    if !mmu::map_user_range_for_root(root_ppn, PROGRAM_VA_BASE, PROGRAM_WINDOW_BYTES, perms) {
        logf!("launch_program: mapping failed (root=0x%x)", root_ppn);
        return None;
    }

    // Copy code and arguments into user memory.
    if !mmu::copy_into_user(root_ppn, Config::PROGRAM_START_ADDR, code) {
        logf!("launch_program: failed to copy code into root=0x%x", root_ppn);
        return None;
    }
    if !mmu::copy_into_user(root_ppn, TO_PTR_ADDR, &to.0) {
        logf!("launch_program: failed to copy 'to' address into root=0x%x", root_ppn);
        return None;
    }
    if !mmu::copy_into_user(root_ppn, FROM_PTR_ADDR, &from.0) {
        logf!("launch_program: failed to copy 'from' address into root=0x%x", root_ppn);
        return None;
    }
    if !mmu::copy_into_user(root_ppn, INPUT_BASE_ADDR, input) {
        logf!("launch_program: failed to copy input into root=0x%x", root_ppn);
        return None;
    }

    let mut task = Task::new(AddressSpace::new(root_ppn, asid), kstack_top);
    // Set up initial trapframe.
    let stack_top = PROGRAM_VA_BASE
        .wrapping_add((Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT + STACK_BYTES) as u32);
    task.tf.pc = Config::PROGRAM_START_ADDR;
    task.tf.regs[REG_SP] = stack_top;
    task.tf.regs[REG_A0] = TO_PTR_ADDR;
    task.tf.regs[REG_A1] = FROM_PTR_ADDR;
    task.tf.regs[REG_A2] = INPUT_BASE_ADDR;
    task.tf.regs[REG_A3] = input.len() as u32;

    Some(task)
}

fn alloc_asid() -> u16 {
    unsafe {
        let counter = NEXT_ASID.get_mut();
        let asid = if *counter == 0 { 1 } else { *counter };
        *counter = asid.wrapping_add(1);
        asid
    }
}
