use alloc::vec;
use core::slice;

use program::{log, logf};
use state::State;

use kernel::global::{BOOT_INFO, PAGE_ALLOC_INIT, STATE, TASKS};
use kernel::{mmu, BootInfo, Task, trap};

/// Initialize kernel state from the bootloader handoff and optional state blob.
pub fn init_kernel(state_ptr: *const u8, state_len: usize, boot_info_ptr: *const BootInfo) {
    if let Some(info) = unsafe { boot_info_ptr.as_ref() } {
        trap::init_trap_vector(info.kstack_top);
    }
    init_state(state_ptr, state_len);
    init_boot_info(boot_info_ptr);
}

fn init_state(state_ptr: *const u8, state_len: usize) {
    logf!(
        "init_state: state_ptr=0x%x state_len=%d",
        state_ptr as usize as u32,
        state_len as u32
    );
    unsafe {
        let state_slot = STATE.get_mut();
        if !state_ptr.is_null() && state_len > 0 {
            let bytes = slice::from_raw_parts(state_ptr, state_len);
            *state_slot = State::decode(bytes).or_else(|| {
                log!("state decode failed; starting empty state");
                Some(State::new())
            });
            if state_slot.is_some() {
                logf!("state initialized (len=%d)", state_len as u32);
            }
        } else {
            *state_slot = Some(State::new());
        }
    }
}

fn init_boot_info(boot_info_ptr: *const BootInfo) {
    logf!(
        "init_boot_info: boot_info_ptr=0x%x",
        boot_info_ptr as usize as u32
    );
    if let Some(info) = unsafe { boot_info_ptr.as_ref() } {
        unsafe {
            *BOOT_INFO.get_mut() = Some(*info);
        }
        unsafe {
            if !*PAGE_ALLOC_INIT.get_mut() {
                mmu::init(info);
                *PAGE_ALLOC_INIT.get_mut() = true;
            }
        }
        let task = Task::kernel(info.root_ppn, info.kstack_top, info.heap_ptr);
        unsafe {
            let tasks_slot = TASKS.get_mut();
            match tasks_slot {
                Some(tasks) => tasks.push(task),
                None => *tasks_slot = Some(vec![task]),
            }
        }
        logf!(
            "boot_info: root_ppn=0x%x kstack_top=0x%x heap_ptr=0x%x mem_size=%d",
            info.root_ppn,
            info.kstack_top,
            info.heap_ptr,
            info.memory_size
        );
    } else {
        log!("boot_info missing; kernel task not initialized");
    }
}
