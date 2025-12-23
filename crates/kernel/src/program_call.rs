use alloc::{format, vec};

use kernel::{prep_program_task, run_task, Config, PROGRAM_WINDOW_BYTES};
use kernel::global::{BOOT_INFO, STATE, TASKS};
use program::{log, logf};
use state::State;
use types::transaction::Transaction;

pub(crate) fn program_call(tx: &Transaction) {
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

    let first_nz = account
        .code
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(account.code.len());
    let nz_count = account.code.iter().filter(|&&b| b != 0).count();
    logf!(
        "%s",
        display: format!(
            "Program code stats: len={} first_nz={} nz_count={}",
            account.code.len(),
            first_nz,
            nz_count
        )
    );

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

    let entry_off = first_nz as u32;

    if let Some(task) =
        prep_program_task(kstack_top, &tx.to, &tx.from, &account.code, &tx.data, entry_off)
    {
        logf!(
            "Program task created: root=0x%x asid=%d window_size=%d",
            task.addr_space.root_ppn,
            task.addr_space.asid as u32,
            PROGRAM_WINDOW_BYTES as u32
        );
        unsafe {
            let tasks_slot = TASKS.get_mut();
            match tasks_slot {
                Some(tasks) => tasks.push(task),
                None => *tasks_slot = Some(vec![task]),
            }
            if let Some(tasks) = tasks_slot {
                if let Some(last) = tasks.last() {
                    run_task(last);
                }
            }
        }
    } else {
        log!("Program call skipped: no memory manager installed");
    }
}
