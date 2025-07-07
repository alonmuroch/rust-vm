mod tests;

use avm::transaction::{TransactionType};
use types::address::Address;
use avm::avm::AVM;
use state::State;

use crate::tests::TEST_CASES;

pub const VM_MEMORY_SIZE: usize = 15 * 1024; // 15 KB
pub const MAX_MEMORY_PAGES: usize = 10; 

#[test]
fn test_entrypoint_function() {
    for case in TEST_CASES.iter() {
        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        for tx in transactions {
            // Log the transaction details
            println!(
                "Running {:?} tx:\n  From: {:?}\n  To: {:?}\n  Data len: {:?}",
                tx.tx_type, tx.from, tx.to, tx.data.len()
            );

            let res = avm.run_tx(tx);
            let success = res.success;
            let error_code = res.error_code;
            // avm.state.pretty_print();
            // avm.memory_manager.dump_all_pages_linear();
            println!("Success: {}, Error code: {}\n", success, error_code);
        }
    }
}


