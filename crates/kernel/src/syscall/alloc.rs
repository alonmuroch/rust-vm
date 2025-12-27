use program::{log, logf};

use crate::global::{CURRENT_TASK, TASKS};
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
    logf!(
        "sys_alloc: size=0x%x align=0x%x heap_ptr=0x%x task=%d",
        size,
        align,
        task.heap_ptr,
        current as u32
    );

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

    let window_base = task.addr_space.va_base;
    let window_limit = window_base.saturating_add(task.addr_space.va_len);
    if start < window_base || end > window_limit {
        logf!(
            "sys_alloc: heap range exceeds task window start=0x%x end=0x%x window=[0x%x,0x%x)",
            start,
            end,
            window_base,
            window_limit
        );
        return 0;
    }
    task.heap_ptr = end;
    start
}

pub(crate) fn sys_dealloc(_args: [u32; 6]) -> u32 {
    // No-op: kernel heap is bump-only for now.
    0
}
