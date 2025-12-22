#![allow(dead_code)]

use crate::{AddressSpace, Config, Task};
use program::logf;

// Linux-style protection and mapping flags (subset).
const PROT_READ: u32 = 0x1;
const PROT_WRITE: u32 = 0x2;
const PROT_EXEC: u32 = 0x4;

const MAP_PRIVATE: u32 = 0x02;
const MAP_ANON: u32 = 0x20;
const MAP_FIXED: u32 = 0x10;

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

fn syscall_mmap(addr: u32, len: usize, prot: u32, flags: u32) -> u32 {
    let ret: u32;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") 222u32,        // __NR_mmap
            inlateout("a0") addr => ret,
            in("a1") len as u32,
            in("a2") prot,
            in("a3") flags,
            in("a4") 0u32,          // fd
            in("a5") 0u32,          // offset
        );
    }
    ret
}

/// Create a new task for a program and map its virtual address window via syscalls.
///
/// This sets up:
/// - Maps a fixed VA window [PROGRAM_VA_BASE, PROGRAM_VA_BASE + PROGRAM_WINDOW_BYTES).
/// - Returns a Task with the new address space and provided kernel stack top.
///
/// The caller is responsible for copying program bytes into the mapped window
/// and initializing the user trapframe (PC/SP/args) before running.
pub fn launch_program(asid: u16, kstack_top: u32) -> Option<Task> {
    let prot = PROT_READ | PROT_WRITE | PROT_EXEC;
    let flags = MAP_PRIVATE | MAP_ANON | MAP_FIXED;
    let addr = syscall_mmap(PROGRAM_VA_BASE, PROGRAM_WINDOW_BYTES, prot, flags);
    if addr != PROGRAM_VA_BASE {
        logf!("launch_program: mmap failed");
        return None;
    }

    // Root/asid tracking is minimal; real satp handling is TBD.
    let task = Task::new(AddressSpace::new(0, asid), kstack_top);
    logf!(
        "launch_program: asid=%d base=0x%x size=%d",
        asid as u32,
        PROGRAM_VA_BASE,
        PROGRAM_WINDOW_BYTES as u32
    );
    Some(task)
}
