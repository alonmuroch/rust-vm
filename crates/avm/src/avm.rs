use crate::memory_page_manager::MemoryPageManager;
use storage::Storage;
use vm::vm::VM;
use vm::memory_page::MemoryPage;
use vm::registers::Register;
use state::State;
use crate::transaction::Transaction;
use crate::global::Config;
use crate::execution_context::ExecutionContext;
use types::address::Address;
use types::result::Result;
use std::{panic::{catch_unwind, AssertUnwindSafe}, usize};
use std::rc::Rc;

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
        let result_ptr = if self.state.is_contract(tx.to) {
            self.call_contract(tx.from, tx.to, tx.data)
        } else {
            panic!("Entrypoint: destination address {} is not a contract", tx.to);  
        };

        return self.extract_result(result_ptr);
    }

    fn extract_result(& self, result_ptr: u32) -> Result {
        let page = self.memory_manager.first_page().expect("No memory page allocated");
        let mem = page.mem();
        let start = result_ptr as usize;

        if start + 5 > mem.len() {
            panic!("Result struct out of bounds at 0x{:08x}", start);
        }

        let error_code = u32::from_le_bytes(mem[start..start + 4].try_into().unwrap());
        let success = mem[start + 4] != 0;

        Result { error_code, success }
    }

    /// Handles calling a new contract, spinning up a fresh VM with its own memory page
    pub fn call_contract(&mut self, from: Address, to: Address, input_data: Vec<u8>) -> u32 {
        // Allocate a fresh memory page for the new VM
        let memory_page: Rc<MemoryPage> = self.memory_manager.new_page();

        // Instantiate and run the child VM
        let mut vm = VM::new(
            memory_page,
        );

        // Set the code for the VM to execute
        let account = self.state.get_account(&to).expect("Contract code not found");
        if account.is_contract == false {
            panic!("destination address {} is not a contract", to);
        }
        let code_slice: &[u8] = &account.code;
        vm.set_code(Config::PROGRAM_START_ADDR, code_slice);  
        vm.cpu.verbose = true;

        // to address
        let _address_ptr: u32 = vm.set_reg_to_data(Register::A0, to.0.as_ref());

        // from address
        let _pubkey_ptr = vm.set_reg_to_data(Register::A1, from.0.as_ref());

        // input data
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

        // result pointer
        let result_ptr = vm.set_reg_to_data(Register::A4, &[0u8; 5]);

        // run the VM
        let result = catch_unwind(AssertUnwindSafe(|| {
            vm.raw_run(); // this might panic
        }));
        if let Err(e) = result {
            eprintln!("💥 VM panicked: {:?}", e);
            panic!("VM panicked");
        }

        // Pop the context when finished
        self.context_stack.pop();

        return result_ptr;
    }

    /// Peek the current active execution context
    pub fn current_context(&self) -> Option<&ExecutionContext> {
        self.context_stack.last()
    }
}
