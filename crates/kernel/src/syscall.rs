//! Kernel-owned syscall stubs. These mirror the bootloader syscalls but
//! are now dispatched from the kernel trap handler. Implementations will
//! land here; for now they panic to make missing pieces explicit.
use program::{log, logf};

use crate::global::{CURRENT_TASK, TASKS};

pub const SYSCALL_STORAGE_GET: u32 = 1;
pub const SYSCALL_STORAGE_SET: u32 = 2;
pub const SYSCALL_PANIC: u32 = 3;
pub const SYSCALL_LOG: u32 = 100;
pub const SYSCALL_CALL_PROGRAM: u32 = 5;
pub const SYSCALL_FIRE_EVENT: u32 = 6;
pub const SYSCALL_ALLOC: u32 = 7;
pub const SYSCALL_DEALLOC: u32 = 8;
pub const SYSCALL_TRANSFER: u32 = 9;
pub const SYSCALL_BALANCE: u32 = 10;
pub const SYSCALL_BRK: u32 = 214;

pub fn dispatch_syscall(call_id: u32, args: [u32; 6]) -> u32 {
    match call_id {
        SYSCALL_STORAGE_GET => sys_storage_get(args),
        SYSCALL_STORAGE_SET => sys_storage_set(args),
        SYSCALL_PANIC => sys_panic(args),
        SYSCALL_LOG => sys_log(args),
        SYSCALL_CALL_PROGRAM => sys_call_program(args),
        SYSCALL_FIRE_EVENT => sys_fire_event(args),
        SYSCALL_ALLOC => sys_alloc(args),
        SYSCALL_DEALLOC => sys_dealloc(args),
        SYSCALL_TRANSFER => sys_transfer(args),
        SYSCALL_BALANCE => sys_balance(args),
        SYSCALL_BRK => sys_brk(args),
        _ => {
            logf!("unknown syscall id %d", call_id);
            0
        }
    }
}

fn sys_storage_get(_args: [u32; 6]) -> u32 {
    log!("sys_storage_get: need implementation");
    0
}

fn sys_storage_set(_args: [u32; 6]) -> u32 {
    log!("sys_storage_set: need implementation");
    0
}

fn sys_panic(_args: [u32; 6]) -> u32 {
    log!("sys_panic: need implementation");
    0
}

fn sys_log(_args: [u32; 6]) -> u32 {
    log!("sys_log: need implementation");
    0
}

fn sys_call_program(_args: [u32; 6]) -> u32 {
    log!("sys_call_program: need implementation");
    0
}

fn sys_fire_event(_args: [u32; 6]) -> u32 {
    log!("sys_fire_event: need implementation");
    0
}

fn sys_alloc(args: [u32; 6]) -> u32 {
    let size = args[0];
    let align = args[1];

    if size == 0 {
        log!("sys_alloc: invalid size 0");
        return 0;
    }
    if align == 0 || (align & (align - 1)) != 0 {
        logf!("sys_alloc: invalid alignment %d", align);
        return 0;
    }

    let current = unsafe { *CURRENT_TASK.get_mut() };
    let tasks = unsafe { TASKS.get_mut() };
    let task = match tasks.get_mut(current) {
        Some(task) => task,
        None => {
            logf!("sys_alloc: no current task for slot %d", current as u32);
            return 0;
        }
    };

    let mask = align - 1;
    let start = match task.heap_ptr.checked_add(mask) {
        Some(addr) => addr & !mask,
        None => {
            log!("sys_alloc: heap ptr overflow");
            return 0;
        }
    };
    let end = match start.checked_add(size) {
        Some(end) => end,
        None => {
            log!("sys_alloc: size overflow");
            return 0;
        }
    };

    task.heap_ptr = end;
    start
}

fn sys_dealloc(_args: [u32; 6]) -> u32 {
    log!("sys_dealloc: need implementation");
    0
}

fn sys_transfer(_args: [u32; 6]) -> u32 {
    log!("sys_transfer: need implementation");
    0
}

fn sys_balance(_args: [u32; 6]) -> u32 {
    log!("sys_balance: need implementation");
    0
}

fn sys_brk(_args: [u32; 6]) -> u32 {
    log!("sys_brk: need implementation");
    0
}
