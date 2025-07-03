use crate::memory_page_manager::MemoryPageManager;
use storage::Storage;
use vm::vm::VM;
use vm::registers::Register;
use state::{State, Account};
use crate::transaction::{TransactionType, Transaction};
use crate::global::Config;
use crate::execution_context::ExecutionContext;
use types::address::Address;
use types::result::Result;
use std::{panic::{catch_unwind, AssertUnwindSafe}, usize};
use std::rc::Rc;
use core::cell::RefCell;

pub struct AVM {
    /// Stack of execution contexts (for nested contract calls)
    pub context_stack: Vec<ExecutionContext>,

    /// Manages allocation of memory pages
    pub memory_manager: MemoryPageManager,

    /// Global persistent storage
    pub storage: Storage,

    // Global state of the AVM
    pub state: State,
}

impl AVM {
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        Self {
            context_stack: Vec::new(),
            memory_manager: MemoryPageManager::new(max_pages, page_size),
            storage: Storage::new(),
            state: State::new(),
        }
    }

    pub fn run_tx(&mut self, tx: Transaction) -> Result {
        match tx.tx_type {
            TransactionType::Transfer => {
                panic!("regular call not implemented");
            }

            TransactionType::CreateAccount => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    self.create_account(tx.from, tx.to, tx.data);
                }));
                if let Err(e) = result {
                    return Result{error_code: 0, success: false};
                } else {
                    return Result{error_code: 1, success: true};
                }
            }

            TransactionType::ProgramCall => {
                assert!(self.state.is_contract(tx.to), "destination address is not a contract");

                let result_ptr = self.call_contract(tx.from, tx.to, tx.data);
                self.extract_result(result_ptr)
            }
        }
    }


    fn extract_result(&self, result_ptr: u32) -> Result {
        let page_rc = self.memory_manager.first_page().expect("No memory page allocated");

        // Borrow the MemoryPage
        let page = page_rc.borrow();

        // Call the mem() method on the borrowed page
        let mem = page.mem();
        let start = result_ptr as usize;

        if start + 5 > mem.len() {
            panic!("Result struct out of bounds at 0x{:08x}", start);
        }

        let error_code = u32::from_le_bytes(mem[start..start + 4].try_into().unwrap());
        let success = mem[start + 4] != 0;

        Result { error_code, success }
    }

   pub fn create_account(&mut self, _from: Address, to: Address, data: Vec<u8>) {
        // Check that the target address is not already in use
        if self.state.accounts.contains_key(&to) {
            panic!("account already exists");
        }

        // Optional: Charge from `from` account for account creation
        // self.charge_for_deployment(from, &data)?; // not shown here

        // Create and insert new account with code
        let account = Account {
            code: data,
            storage: Default::default(),
            balance: 0,
            nonce: 0,
            is_contract: true,
        };

        self.state.accounts.insert(to, account);
    }


    /// Handles calling a new contract, spinning up a fresh VM with its own memory page
    pub fn call_contract(&mut self, from: Address, to: Address, input_data: Vec<u8>) -> u32 {
        // Get mutable reference to the contract account
        let account = self.state.get_account_mut(&to);
        if !account.is_contract {
            panic!("destination address {} is not a contract", to);
        }

        // Allocate memory and clone storage
        let memory_page = self.memory_manager.new_page();
        let storage = Rc::new(RefCell::new(Storage::with_map(account.storage.clone())));

        // Create and configure child VM
        let mut vm = VM::new(memory_page, storage.clone());
        vm.set_code(Config::PROGRAM_START_ADDR, &account.code);

        // Set registers
        let _address_ptr = vm.set_reg_to_data(Register::A0, to.0.as_ref());
        let _pubkey_ptr = vm.set_reg_to_data(Register::A1, from.0.as_ref());

        let input_len = input_data.len();
        if input_len > Config::MAX_INPUT_LEN {
            panic!(
                "Entrypoint: input length {} exceeds MAX_INPUT_LEN ({})",
                input_len,
                Config::MAX_INPUT_LEN
            );
        }

        let _input_ptr = vm.set_reg_to_data(Register::A2, &input_data);
        vm.set_reg_u32(Register::A3, input_len as u32);

        let result_ptr = vm.set_reg_to_data(Register::A4, &[0u8; 5]);

        // Run the VM safely
        let result = catch_unwind(AssertUnwindSafe(|| {
            vm.raw_run();
        }));

        if let Err(e) = result {
            eprintln!("💥 VM panicked: {:?}", e);
            panic!("VM panicked");
        }

        // Copy storage back into account (clone the updated map)
        let updated_map = storage.borrow().map.borrow().clone();
        account.storage = updated_map;

        // Pop context if using a stack
        self.context_stack.pop();

        result_ptr
    }


    /// Peek the current active execution context
    pub fn current_context(&self) -> Option<&ExecutionContext> {
        self.context_stack.last()
    }
}
