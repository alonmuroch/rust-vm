#![allow(dead_code)]

use std::env;
use std::fs::{self, File};
use std::io::Write as IoWrite;
use std::path::Path;
use std::rc::Rc;

use core::cell::RefCell;
use core::fmt::Write;
use bootloader::bootloader::Bootloader;
use state::State;

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
        self.file
            .write_all(s.as_bytes())
            .map_err(|_| core::fmt::Error)?;
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
pub struct TestRunner {
    writer: Rc<RefCell<dyn Write>>,
    verbose: bool,
    vm_memory_size: usize,
    max_memory_pages: usize,
    kernel_bytes: Option<Vec<u8>>,
    kernel_path: Option<String>,
}

impl TestRunner {
    /// Create a new test runner with console output (default)
    pub fn new() -> Self {
        Self::with_writer(Rc::new(RefCell::new(ConsoleWriter)))
    }

    /// Set VM memory size
    pub fn with_memory_size(mut self, size: usize) -> Self {
        self.vm_memory_size = size;
        self
    }

    /// Set max memory pages
    pub fn with_max_pages(mut self, pages: usize) -> Self {
        self.max_memory_pages = pages;
        self
    }

    /// Enable or disable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Create a test runner with file output
    pub fn with_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file_writer = FileWriter::new(path.as_ref().to_str().unwrap())?;
        Ok(Self::with_writer(Rc::new(RefCell::new(file_writer))))
    }

    /// Create a test runner with a custom writer
    pub fn with_writer(writer: Rc<RefCell<dyn Write>>) -> Self {
        TestRunner {
            writer,
            verbose: false,
            vm_memory_size: 512 * 1024, // larger default to accommodate bigger binaries without RVC
            max_memory_pages: 128,      // allow more pages for larger programs
            kernel_bytes: Self::load_kernel_from_env(),
            kernel_path: env::var("KERNEL_ELF").ok(),
        }
    }

    /// Execute all test cases
    pub fn execute(&self) -> Result<(), String> {
        use super::TEST_CASES;

        writeln!(self.writer.borrow_mut(), "=== Starting Test Run ===").unwrap();
        writeln!(
            self.writer.borrow_mut(),
            "Verbose logging: {}",
            if self.verbose { "enabled" } else { "disabled" }
        )
        .unwrap();

        for case in TEST_CASES.iter() {
            self.run_test_case(case)?;
        }

        // Write test summary
        writeln!(self.writer.borrow_mut(), "\n=== Test Run Complete ===").unwrap();
        writeln!(
            self.writer.borrow_mut(),
            "Total test cases: {}",
            TEST_CASES.len()
        )
        .unwrap();

        Ok(())
    }

    /// Run a single test case
    fn run_test_case(&self, case: &super::TestCase) -> Result<(), String> {
        let mut bootloader = Bootloader::new(self.max_memory_pages, self.vm_memory_size);
        let state = Rc::new(RefCell::new(State::new()));

        // Write test case header
        writeln!(
            self.writer.borrow_mut(),
            "\n############################################"
        )
        .unwrap();
        writeln!(
            self.writer.borrow_mut(),
            "#### Running test case: {} ####",
            case.name
        )
        .unwrap();
        writeln!(
            self.writer.borrow_mut(),
            "############################################"
        )
        .unwrap();

        // Print address to binary mappings
        if !case.address_mappings.is_empty() {
            writeln!(self.writer.borrow_mut(), "\n📍 Address -> Binary Mappings:").unwrap();
            for (addr, binary) in &case.address_mappings {
                writeln!(self.writer.borrow_mut(), "  {} -> {}", addr, binary).unwrap();
            }
        }
        writeln!(self.writer.borrow_mut()).unwrap();

        // Execute the whole bundle via the bootloader/kernel path.
        bootloader.execute_bundle(
            self.kernel_bytes.as_ref().ok_or_else(|| {
                "KERNEL_ELF not set or unreadable; bootloader path required".to_string()
            })?,
            &case.bundle,
            state,
        );

        // For now we treat successful bootloader execution as a passed test.
        Ok(())
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::with_writer(Rc::new(RefCell::new(ConsoleWriter)))
    }
}

impl TestRunner {
    fn load_kernel_from_env() -> Option<Vec<u8>> {
        let path =
            env::var("KERNEL_ELF").unwrap_or_else(|_| "crates/bootloader/bin/kernel.elf".to_string());
        fs::read(&path).ok()
    }
}
