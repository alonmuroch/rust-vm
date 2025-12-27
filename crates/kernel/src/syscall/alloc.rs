use program::{log, logf};

use crate::global::{CURRENT_TASK, TASKS};
use crate::mmu;

pub(crate) fn sys_alloc(args: [u32; 6]) -> u32 {
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

    let len = end.saturating_sub(start) as usize;
    if !mmu::map_kernel_range(start, len, mmu::PagePerms::kernel_rw()) {
        log!("sys_alloc: failed to map heap range");
        return 0;
    }
    task.heap_ptr = end;
    start
}

pub(crate) fn sys_dealloc(_args: [u32; 6]) -> u32 {
    // No-op: kernel heap is bump-only for now.
    0
}
