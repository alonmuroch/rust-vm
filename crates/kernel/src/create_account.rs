use alloc::format;

use kernel::global::STATE;
use kernel::Config;
use program::log;
use state::State;
use types::transaction::Transaction;

pub(crate) fn create_account(tx: &Transaction) {
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
