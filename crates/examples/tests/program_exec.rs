mod tests;

use avm::avm::AVM;
use crate::tests::TEST_CASES;

pub const VM_MEMORY_SIZE: usize = 64 * 1024; // 64 KB - increased to support larger programs with external libraries
pub const MAX_MEMORY_PAGES: usize = 20;  // Increased memory pages 

#[test]
fn test_entrypoint_function() {
    for case in TEST_CASES.iter() {
        println!("\n############################################");
        println!("#### Running test case: {} ####", case.name);
        println!("############################################\n");

        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        // avm.set_verbosity(true);
        let mut last_success: bool = false;
        let mut last_error_code: u32 = 0;
        let mut last_result: Option<types::Result> = None;
        for tx in transactions {
            // Log the transaction details
            println!(
                "Running {:?} tx:\n  From: {:?}\n  To: {:?}\n  Data len: {:?}",
                tx.tx_type, tx.from, tx.to, tx.data.len()
            );

            let receipt = avm.run_tx(tx);
            last_success = receipt.result.success;
            last_error_code = receipt.result.error_code;
            last_result = Some(receipt.result.clone());
            avm.state.pretty_print();
            // avm.memory_manager.dump_all_pages_linear();

            if let Some(abi) = &case.abi {
                receipt.print_events_pretty(abi);
                println!("{}", receipt);
            } else {
                println!("{}", receipt);
            }
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
        
        // Check expected data if specified
        if let Some(expected_data) = &case.expected_data {
            if let Some(result) = last_result {
                let actual_data = &result.data[..result.data_len as usize];
                assert_eq!(
                    actual_data, expected_data.as_slice(),
                    "{}: expected equal data",
                    case.name
                );
            }
        }
    }
}


