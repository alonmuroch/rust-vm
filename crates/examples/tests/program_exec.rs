mod tests;

use avm::avm::AVM;
use crate::tests::TEST_CASES;
use std::rc::Rc;
use core::cell::RefCell;
use core::fmt::Write;
use std::fs::File;
use std::io::Write as IoWrite;

// File writer for logging to disk
struct FileWriter {
    file: File,
}

impl FileWriter {
    fn new(path: &str) -> std::io::Result<Self> {
        Ok(FileWriter {
            file: File::create(path)?,
        })
    }
}

impl Write for FileWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.file.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)?;
        self.file.flush().map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}

pub const VM_MEMORY_SIZE: usize = 64 * 1024; // 64 KB - increased to support larger programs with external libraries
pub const MAX_MEMORY_PAGES: usize = 20;  // Increased memory pages
pub const VERBOSE_LOGGING: bool = true;

/// Console writer that wraps println!
struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

#[test]
fn test_entrypoint_function() {
    // Set up logging once for all test cases
    // Choose your logging mode by uncommenting ONE of the lines below:
    
    // Option 1: Log to console with verbose output
    // let writer: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(ConsoleWriter));
    
    // Option 2: Log to file with verbose output  
    let log_path = "/Users/alonmuroch/Desktop/logs.txt";
    let writer: Rc<RefCell<dyn Write>> = match FileWriter::new(log_path) {
        Ok(file_writer) => {
            eprintln!("📝 All output will be written to: {}", log_path);
            Rc::new(RefCell::new(file_writer))
        }
        Err(e) => {
            eprintln!("⚠️ Failed to create log file at {}: {}", log_path, e);
            eprintln!("   Falling back to console output");
            Rc::new(RefCell::new(ConsoleWriter))
        }
    };
    
    writeln!(writer.borrow_mut(), "=== Starting Test Run ===").unwrap();
    writeln!(writer.borrow_mut(), "Verbose logging: {}", if VERBOSE_LOGGING { "enabled" } else { "disabled" }).unwrap();
    
    for case in TEST_CASES.iter() {
        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(MAX_MEMORY_PAGES, VM_MEMORY_SIZE);
        
        // Set up AVM with the chosen writer and verbosity
        avm.set_verbosity(VERBOSE_LOGGING);
        avm.set_verbose_writer(writer.clone());
        
        // Write test case header using the configured writer
        writeln!(writer.borrow_mut(), "\n############################################").unwrap();
        writeln!(writer.borrow_mut(), "#### Running test case: {} ####", case.name).unwrap();
        writeln!(writer.borrow_mut(), "############################################\n").unwrap();
        
        let mut last_success: bool = false;
        let mut last_error_code: u32 = 0;
        let mut last_result: Option<types::Result> = None;
        for tx in transactions {
            // Log the transaction details using the configured writer
            writeln!(writer.borrow_mut(),
                "Running {:?} tx:\n  From: {:?}\n  To: {:?}\n  Data len: {:?}",
                tx.tx_type, tx.from, tx.to, tx.data.len()
            ).unwrap();

            let receipt = avm.run_tx(tx);
            last_success = receipt.result.success;
            last_error_code = receipt.result.error_code;
            last_result = Some(receipt.result.clone());
            
            // Write state dump to the configured writer
            writeln!(writer.borrow_mut(), "--- State Dump ---").unwrap();
            for (address, account) in &avm.state.accounts {
                writeln!(writer.borrow_mut(), "  🔑 Address: 0x{}", address).unwrap();
                writeln!(writer.borrow_mut(), "      - Balance: {}", account.balance).unwrap();
                writeln!(writer.borrow_mut(), "      - Nonce: {}", account.nonce).unwrap();
                writeln!(writer.borrow_mut(), "      - Is contract?: {}", account.is_contract).unwrap();
                writeln!(writer.borrow_mut(), "      - Code size: {} bytes", account.code.len()).unwrap();
                writeln!(writer.borrow_mut(), "      - Storage:").unwrap();
                for (key, value) in &account.storage {
                    writeln!(writer.borrow_mut(), "        [{:?}] = {:?}", key, value).unwrap();
                }
                writeln!(writer.borrow_mut(), "").unwrap();
            }
            writeln!(writer.borrow_mut(), "--------------------").unwrap();

            // Write receipt to the configured writer
            if let Some(_abi) = &case.abi {
                // TODO: Update print_events_pretty to use the writer
                // For now, just write the receipt
                writeln!(writer.borrow_mut(), "{}", receipt).unwrap();
            } else {
                writeln!(writer.borrow_mut(), "{}", receipt).unwrap();
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
    
    // Write test summary
    writeln!(writer.borrow_mut(), "\n=== Test Run Complete ===").unwrap();
    writeln!(writer.borrow_mut(), "Total test cases: {}", TEST_CASES.len()).unwrap();
}


