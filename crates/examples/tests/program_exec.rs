mod tests;

use avm::avm::AVM;
use crate::tests::TEST_CASES;
use std::rc::Rc;
use core::cell::RefCell;
use core::fmt::Write;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::Path;

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

/// Console writer that wraps println!
struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

/// Test runner that encapsulates test execution with configurable output
struct TestRunner {
    writer: Rc<RefCell<dyn Write>>,
    verbose: bool,
    vm_memory_size: usize,
    max_memory_pages: usize,
}

impl TestRunner {
    /// Create a new test runner with console output (default)
    fn new() -> Self {
        Self::with_writer(Rc::new(RefCell::new(ConsoleWriter)))
    }

    /// Set VM memory size
    fn with_memory_size(mut self, size: usize) -> Self {
        self.vm_memory_size = size;
        self
    }

    /// Set max memory pages
    fn with_max_pages(mut self, pages: usize) -> Self {
        self.max_memory_pages = pages;
        self
    }

    /// Create a test runner with file output
    fn with_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file_writer = FileWriter::new(path.as_ref().to_str().unwrap())?;
        Ok(Self::with_writer(Rc::new(RefCell::new(file_writer))))
    }

    /// Create a test runner with a custom writer
    fn with_writer(writer: Rc<RefCell<dyn Write>>) -> Self {
        TestRunner {
            writer,
            verbose: false,
            vm_memory_size: 64 * 1024,  // 64KB default
            max_memory_pages: 20,        // 20 pages default
        }
    }

    /// Execute all test cases
    fn execute(&self) -> Result<(), String> {
        writeln!(self.writer.borrow_mut(), "=== Starting Test Run ===").unwrap();
        writeln!(self.writer.borrow_mut(), "Verbose logging: {}", if self.verbose { "enabled" } else { "disabled" }).unwrap();

        for case in TEST_CASES.iter() {
            self.run_test_case(case)?;
        }

        // Write test summary
        writeln!(self.writer.borrow_mut(), "\n=== Test Run Complete ===").unwrap();
        writeln!(self.writer.borrow_mut(), "Total test cases: {}", TEST_CASES.len()).unwrap();

        Ok(())
    }

    /// Run a single test case
    fn run_test_case(&self, case: &crate::tests::TestCase) -> Result<(), String> {
        let transactions = case.bundle.transactions.clone();
        let mut avm = AVM::new(self.max_memory_pages, self.vm_memory_size);

        // Set up AVM with the chosen writer and verbosity
        avm.set_verbosity(self.verbose);
        avm.set_verbose_writer(self.writer.clone());

        // Write test case header
        writeln!(self.writer.borrow_mut(), "\n############################################").unwrap();
        writeln!(self.writer.borrow_mut(), "#### Running test case: {} ####", case.name).unwrap();
        writeln!(self.writer.borrow_mut(), "############################################").unwrap();

        // Print address to binary mappings
        if !case.address_mappings.is_empty() {
            writeln!(self.writer.borrow_mut(), "\n📍 Address -> Binary Mappings:").unwrap();
            for (addr, binary) in &case.address_mappings {
                writeln!(self.writer.borrow_mut(), "  {} -> {}", addr, binary).unwrap();
            }
        }
        writeln!(self.writer.borrow_mut()).unwrap();

        let mut last_success: bool = false;
        let mut last_error_code: u32 = 0;
        let mut last_result: Option<types::Result> = None;

        for tx in transactions {
            // Log the transaction details
            writeln!(self.writer.borrow_mut(),
                "Running {:?} tx:\n  From: {:?}\n  To: {:?}\n  Data len: {:?}",
                tx.tx_type, tx.from, tx.to, tx.data.len()
            ).unwrap();

            let receipt = avm.run_tx(tx);
            last_success = receipt.result.success;
            last_error_code = receipt.result.error_code;
            last_result = Some(receipt.result.clone());

            // Write state dump
            writeln!(self.writer.borrow_mut(), "--- State Dump ---").unwrap();
            for (address, account) in &avm.state.accounts {
                writeln!(self.writer.borrow_mut(), "  🔑 Address: 0x{}", address).unwrap();
                writeln!(self.writer.borrow_mut(), "      - Balance: {}", account.balance).unwrap();
                writeln!(self.writer.borrow_mut(), "      - Nonce: {}", account.nonce).unwrap();
                writeln!(self.writer.borrow_mut(), "      - Is contract?: {}", account.is_contract).unwrap();
                writeln!(self.writer.borrow_mut(), "      - Code size: {} bytes", account.code.len()).unwrap();
                writeln!(self.writer.borrow_mut(), "      - Storage:").unwrap();
                for (key, value) in &account.storage {
                    writeln!(self.writer.borrow_mut(), "        [{:?}] = {:?}", key, value).unwrap();
                }
                writeln!(self.writer.borrow_mut(), "").unwrap();
            }
            writeln!(self.writer.borrow_mut(), "--------------------").unwrap();

            // Write receipt
            if let Some(_abi) = &case.abi {
                // TODO: Update print_events_pretty to use the writer
                // For now, just write the receipt
                writeln!(self.writer.borrow_mut(), "{}", receipt).unwrap();
            } else {
                writeln!(self.writer.borrow_mut(), "{}", receipt).unwrap();
            }
        }

        // Perform assertions
        if last_success != case.expected_success {
            return Err(format!("{}: expected success={}, got={}",
                case.name, case.expected_success, last_success));
        }

        if last_error_code != case.expected_error_code {
            return Err(format!("{}: expected error_code={}, got={}",
                case.name, case.expected_error_code, last_error_code));
        }

        // Check expected data if specified
        if let Some(expected_data) = &case.expected_data {
            if let Some(result) = last_result {
                let actual_data = &result.data[..result.data_len as usize];
                if actual_data != expected_data.as_slice() {
                    return Err(format!("{}: expected data mismatch", case.name));
                }
            }
        }

        Ok(())
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::with_writer(Rc::new(RefCell::new(ConsoleWriter)))
    }
}

#[test]
fn test_entrypoint_function() {
    TestRunner::default().execute().unwrap()
}

#[test]
fn test_with_file_output() {
    // Example test that always writes to a file
    let runner = TestRunner::with_file("/tmp/test_output.txt")
        .expect("Failed to create file writer");
    runner.execute().unwrap();
}


