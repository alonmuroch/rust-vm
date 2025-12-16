//! Minimal allocator used by the guest kernel. For riscv32 it delegates to
//! syscalls 7/8.
#![cfg(target_arch = "riscv32")]

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;

#[derive(Debug)]
pub struct VmAllocator;

unsafe impl GlobalAlloc for VmAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        syscall_alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        syscall_dealloc(ptr, layout.size());
    }
}

unsafe fn syscall_alloc(size: usize, align: usize) -> *mut u8 {
    let mut result: usize;
    asm!(
        "li a7, 7", // SYSCALL_ALLOC
        "ecall",
        in("a1") size,
        in("a2") align,
        out("a0") result,
    );
    result as *mut u8
}

unsafe fn syscall_dealloc(ptr: *mut u8, size: usize) {
    asm!(
        "li a7, 8", // SYSCALL_DEALLOC
        "ecall",
        in("a1") ptr as usize,
        in("a2") size,
        options(nostack, preserves_flags),
    );
}
