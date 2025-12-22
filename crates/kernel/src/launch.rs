#![allow(dead_code)]

use crate::{AddressSpace, Config, Task, mmu};
use program::logf;

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

/// Create a new task for a program and map its virtual address window via syscalls.
///
/// This sets up:
/// - Maps a fixed VA window [PROGRAM_VA_BASE, PROGRAM_VA_BASE + PROGRAM_WINDOW_BYTES).
/// - Returns a Task with the new address space and provided kernel stack top.
///
/// The caller is responsible for copying program bytes into the mapped window
/// and initializing the user trapframe (PC/SP/args) before running.
pub fn launch_program(asid: u16, kstack_top: u32) -> Option<Task> {
    let perms = mmu::PagePerms::user_rwx();
    if !mmu::map_user_range(PROGRAM_VA_BASE, PROGRAM_WINDOW_BYTES, perms) {
        logf!("launch_program: mapping failed");
        return None;
    }

    // Root/asid tracking is minimal; real satp handling is TBD.
    Some(Task::new(AddressSpace::new(0, asid), kstack_top))
}
