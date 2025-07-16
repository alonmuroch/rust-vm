mod tests;

use avm::avm::AVM;
use crate::tests::TEST_CASES;

pub const VM_MEMORY_SIZE: usize = 15 * 1024; // 15 KB
pub const MAX_MEMORY_PAGES: usize = 10; 

#[test]
fn test_entrypoint_function() {
    for case in TEST_CASES.iter() {
        println!("#### Running test case: {} ####", case.name);

        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        // avm.set_verbosity(true);
        let mut last_success: bool = false;
        let mut last_error_code: u32 = 0;
        for tx in transactions {
            // Log the transaction details
            println!(
                "Running {:?} tx:\n  From: {:?}\n  To: {:?}\n  Data len: {:?}",
                tx.tx_type, tx.from, tx.to, tx.data.len()
            );

            let receipt = avm.run_tx(tx);
            last_success = receipt.result.success;
            last_error_code = receipt.result.error_code;
            // avm.state.pretty_print();
            // avm.memory_manager.dump_all_pages_linear();
            println!("{}", receipt);
        }
        assert_eq!(
            last_success, case.expected_success,
            "{}: expected equal success",
            case.name
        );
        assert_eq!(
            last_error_code, case.expected_error_code,
            "{}: expected equal error code",
            case.name
        );
    }
}


