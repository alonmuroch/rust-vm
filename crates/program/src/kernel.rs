#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use core::mem::forget;
use core::slice;
use program::{log, logf, Config};
use state::{Account, State};
use types::transaction::{Transaction, TransactionBundle, TransactionType};

const SYSCALL_COMMIT_STATE: u32 = 11;

/// Kernel entrypoint. Receives a pointer/length pair to an encoded `TransactionBundle`
/// (produced by the bootloader) and walks each transaction. It also receives an
/// encoded state blob that it updates and commits back to the host.
#[unsafe(no_mangle)]
pub extern "C" fn _start(
    bundle_ptr: *const u8,
    bundle_len: usize,
    state_ptr: *const u8,
    state_len: usize,
) {
    // Copy args to locals before any syscalls (ecall clobbers a0).
    let bundle_ptr = bundle_ptr;
    let bundle_len = bundle_len;
    let state_ptr = state_ptr;
    let state_len = state_len;

    log!("kernel boot");
    logf!("bundle_len=%d", bundle_len as u32);

    let encoded_bundle = unsafe { slice::from_raw_parts(bundle_ptr, bundle_len) };
    let encoded_state = unsafe { slice::from_raw_parts(state_ptr, state_len) };

    let mut state = State::decode(encoded_state).unwrap_or_else(State::new);

    if let Some(bundle) = TransactionBundle::decode(encoded_bundle) {
        let count = bundle.transactions.len();
        logf!("decoded tx count=%d", count as u32);
        for i in 0..count {
            logf!("processing tx %d/%d", (i + 1) as u32, count as u32);
            if let Some(tx) = bundle.transactions.get(i) {
                execute_transaction(&mut state, tx);
            } else {
                logf!("missing tx at index %d", i as u32);
            }
        }
        // Avoid drop-time teardown that can allocate/deallocate; we halt immediately.
        forget(bundle);
    } else {
        log!("bundle decode failed");
    }

    let mut encoded_state = state.encode();
    let state_ptr = encoded_state.as_mut_ptr();
    let state_len = encoded_state.len() as u32;
    forget(encoded_state);
    log!("finished bundle execution");
    finish(state_ptr as u32, state_len);
}

fn execute_transaction(state: &mut State, tx: &Transaction) {
    match tx.tx_type {
        TransactionType::CreateAccount => create_account(state, tx),
        _ => log!("executing transaction"),
    }
}

fn create_account(state: &mut State, tx: &Transaction) {
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

    if state.accounts.contains_key(&tx.to) {
        log!("account already exists");
        return;
    }

    let code = tx.data.clone();

    state.accounts.insert(
        tx.to,
        Account {
            nonce: 0,
            balance: 0,
            code,
            is_contract,
            storage: Default::default(),
        },
    );
    log!("account created");
}

#[inline(never)]
fn finish(state_ptr: u32, state_len: u32) -> ! {
    // Persist state back to the host, then halt.
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!(
            "li a7, {commit}",
            "ecall",
            in("a1") state_ptr,
            in("a2") state_len,
            commit = const SYSCALL_COMMIT_STATE,
        );
    }

    unsafe { core::arch::asm!("ebreak") };
    loop {}
}
