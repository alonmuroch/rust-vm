#![no_std]
#![no_main]

extern crate alloc;

use alloc::{format, vec, vec::Vec};
use core::mem::forget;
use core::ptr;
use core::slice;
use kernel::{BootInfo, Config, Task, launch_program, mmu, PROGRAM_WINDOW_BYTES};
use program::{log, logf};
use state::State;
use types::transaction::{Transaction, TransactionBundle, TransactionType};

mod global;
use crate::global::Global;

#[allow(dead_code)]
const KERNEL_TASK_IDX: usize = 0;

#[allow(dead_code)]
static TASKS: Global<Option<Vec<Task>>> = Global::new(None);
static STATE: Global<Option<State>> = Global::new(None);
static BOOT_INFO: Global<Option<BootInfo>> = Global::new(None);
static PAGE_ALLOC_INIT: Global<bool> = Global::new(false);

/// Kernel entrypoint. Receives:
/// - `bundle_ptr`/`bundle_len`: encoded `TransactionBundle` prepared by the bootloader.
/// - `state_ptr`/`state_len`: optional state blob (currently unused).
/// - `boot_info_ptr`: bootloader handoff with stack + page-table root info.
#[unsafe(no_mangle)]
pub extern "C" fn _start(
    bundle_ptr: *const u8,
    bundle_len: usize,
    state_ptr: *const u8,
    state_len: usize,
    boot_info_ptr: *const BootInfo,
) {
    log!("kernel boot");
    logf!("bundle_len=%d", bundle_len as u32);

    // Initialize state from provided blob if present.
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
        let task = Task::kernel(info.root_ppn, info.kstack_top);
        unsafe {
            let tasks_slot = TASKS.get_mut();
            match tasks_slot {
                Some(tasks) => tasks.push(task),
                None => *tasks_slot = Some(vec![task]),
            }
        }
        logf!(
            "boot_info: root_ppn=0x%x kstack_top=0x%x mem_size=%d",
            info.root_ppn,
            info.kstack_top,
            info.memory_size
        );
    } else {
        log!("boot_info missing; kernel task not initialized");
    }

    let encoded_bundle = unsafe { slice::from_raw_parts(bundle_ptr, bundle_len) };

    if let Some(bundle) = TransactionBundle::decode(encoded_bundle) {
        let count = bundle.transactions.len();
        logf!("decoded tx count=%d", count as u32);
        for i in 0..count {
            logf!("processing tx %d/%d", (i + 1) as u32, count as u32);
            if let Some(tx) = bundle.transactions.get(i) {
                execute_transaction(tx);
            } else {
                logf!("missing tx at index %d", i as u32);
            }
        }
        // Avoid drop-time teardown that can allocate/deallocate; we halt immediately.
        forget(bundle);
    } else {
        log!("bundle decode failed");
    }

    log!("finished bundle execution");
    halt();
}

fn execute_transaction(tx: &Transaction) {
    match tx.tx_type {
        TransactionType::CreateAccount => create_account(tx),
        TransactionType::ProgramCall => program_call(tx),
        _ => log!("executing transaction"),
    }
}

fn create_account(tx: &Transaction) {
    let code_size = tx.data.len();
    let is_contract = code_size > 0;

    let msg = format!(
        "Tx creating account at address {}. Is contract: {}. Code size: {} bytes.",
        tx.to, is_contract, code_size
    );
    let msg_ref: &str = msg.as_str();
    log!(msg_ref);

    let max = Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT;
    if code_size > max {
        panic!(
            "❌ Code size ({}) exceeds CODE_SIZE_LIMIT ({} bytes)",
            code_size, max
        );
    }

    let state = unsafe { STATE.get_mut().get_or_insert_with(State::new) };
    let account = state.get_account_mut(&tx.to);
    account.code = tx.data.clone();
    account.is_contract = is_contract;
    let msg = format!(
        "account created in kernel state: addr={} is_contract={} code_len={}",
        tx.to, is_contract, code_size
    );
    let msg_ref: &str = msg.as_str();
    log!(msg_ref);
}

fn program_call(tx: &Transaction) {
    let state = unsafe { STATE.get_mut().get_or_insert_with(State::new) };
    let account = match state.get_account(&tx.to) {
        Some(acc) => acc,
        None => {
            logf!(
                "%s",
                display: format!("Program call failed: account {} does not exist", tx.to)
            );
            return;
        }
    };

    if !account.is_contract {
        logf!(
            "%s",
            display: format!(
                "Program call failed: target {} is not a contract (code_len={})",
                tx.to,
                account.code.len()
            )
        );
        return;
    }

    let code_len = account.code.len();
    let max = Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT;
    if code_len > max {
        panic!(
            "❌ Program call rejected: code size ({}) exceeds limit ({})",
            code_len, max
        );
    }

    logf!(
        "%s",
        display: format!(
            "Program call: from={} to={} input_len={} code_len={}",
            tx.from,
            tx.to,
            tx.data.len(),
            code_len
        )
    );

    let kstack_top = unsafe { BOOT_INFO.get_mut().as_ref().map(|b| b.kstack_top).unwrap_or(0) };

    if let Some(task) = launch_program(0, kstack_top) {
        logf!(
            "Program task created: root=0x%x window_size=%d",
            task.addr_space.root_ppn,
            PROGRAM_WINDOW_BYTES as u32
        );
        unsafe {
            let tasks_slot = TASKS.get_mut();
            match tasks_slot {
                Some(tasks) => tasks.push(task),
                None => *tasks_slot = Some(vec![task]),
            }
        }
    } else {
        log!("Program call skipped: no memory manager installed");
    }
}

#[inline(never)]
fn halt() -> ! {
    unsafe { core::arch::asm!("ebreak") };
    loop {}
}
