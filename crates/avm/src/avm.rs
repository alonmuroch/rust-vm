use crate::memory_page_manager::MemoryPageManager;
use storage::Storage;
use vm::vm::VM;
use vm::registers::Register;
use state::{State, Account};
use crate::transaction::{TransactionType, Transaction};
use crate::global::Config;
use crate::execution_context::{ExecutionContext, ContextStack};
use crate::host_interface::HostShim;
use types::address::Address;
use types::result::Result;
use std::{panic::{catch_unwind, AssertUnwindSafe}, usize};
use std::rc::Rc;
use core::cell::RefCell;

/// Application Virtual Machine (AVM) - the main orchestrator for smart contract execution.
/// 
/// EDUCATIONAL PURPOSE: This struct represents a complete blockchain virtual machine
/// that can execute smart contracts. It's similar to Ethereum's EVM or other blockchain VMs.
/// 
/// AVM ARCHITECTURE OVERVIEW:
/// - Context Stack: Manages nested contract calls (like a call stack in programming)
/// - Memory Manager: Allocates and manages memory pages for contract execution
/// - Storage: Global persistent storage shared across all contracts
/// - State: Manages accounts, balances, and contract code
/// 
/// BLOCKCHAIN CONCEPTS:
/// - Each contract has its own account with code and storage
/// - Transactions can create accounts or call existing contracts
/// - Contracts can call other contracts (nested execution)
/// - All state changes are atomic (all succeed or all fail)
/// 
/// REAL-WORLD BLOCKCHAIN COMPARISON:
/// This AVM is inspired by Ethereum's EVM but simplified for educational purposes:
/// - Ethereum has more complex gas accounting and pricing
/// - Real blockchains have more sophisticated memory management
/// - Production VMs include additional security features like reentrancy protection
/// - Gas limits and execution timeouts prevent infinite loops
/// 
/// VIRTUAL MACHINE LAYERS:
/// The AVM operates at multiple abstraction levels:
/// 1. Transaction Layer: Processes blockchain transactions
/// 2. Contract Layer: Executes smart contract bytecode
/// 3. Memory Layer: Manages contract memory allocation
/// 4. Storage Layer: Provides persistent data storage
/// 5. State Layer: Maintains global blockchain state
/// 
/// SECURITY CONSIDERATIONS:
/// - Panic handling prevents one bad contract from crashing the entire system
/// - Memory isolation between contracts prevents interference
/// - Input validation prevents resource exhaustion attacks
/// - Context tracking prevents unauthorized cross-contract access
#[derive(Debug)]
pub struct AVM {
    /// Stack of execution contexts for nested contract calls.
    /// 
    /// EDUCATIONAL: This implements a call stack similar to how functions
    /// call other functions in programming. Each context tracks who called
    /// whom and with what data. This is crucial for debugging and gas accounting.
    pub context_stack: ContextStack,

    /// Manages allocation of memory pages for contract execution.
    /// 
    /// EDUCATIONAL: Each contract gets its own memory page to prevent
    /// interference between contracts. This is like process isolation in
    /// operating systems - one process can't access another's memory.
    pub memory_manager: MemoryPageManager,

    /// Global persistent storage shared across all contracts.
    /// 
    /// EDUCATIONAL: This is like a global database that persists between
    /// transactions. Contracts can read and write to this storage, and
    /// changes survive across multiple contract calls.
    pub storage: Storage,

    /// Global state of the AVM including all accounts and their data.
    /// 
    /// EDUCATIONAL: This represents the entire blockchain state - all
    /// accounts, their balances, code, and storage. Every transaction
    /// can potentially modify this state.
    pub state: State,
}

impl AVM {
    /// Creates a new Application Virtual Machine with specified memory constraints.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates VM initialization with resource limits.
    /// In blockchain systems, resource limits are crucial to prevent denial-of-service
    /// attacks and ensure predictable execution costs.
    /// 
    /// RESOURCE MANAGEMENT:
    /// - max_pages: Maximum number of memory pages that can be allocated
    /// - page_size: Size of each memory page in bytes
    /// 
    /// INITIALIZATION: All components start in a clean state, ready to
    /// process transactions and execute contracts.
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        Self {
            context_stack: ContextStack::new(),
            memory_manager: MemoryPageManager::new(max_pages, page_size),
            storage: Storage::new(),
            state: State::new(),
        }
    }

    /// Executes a transaction, which can be a transfer, account creation, or contract call.
    /// 
    /// EDUCATIONAL PURPOSE: This is the main entry point for processing blockchain
    /// transactions. Each transaction type has different semantics and security considerations.
    /// 
    /// TRANSACTION TYPES:
    /// - Transfer: Move value between accounts (not implemented in this VM)
    /// - CreateAccount: Deploy a new smart contract
    /// - ProgramCall: Execute an existing smart contract
    /// 
    /// TRANSACTION PROCESSING FLOW:
    /// 1. Validate transaction format and parameters
    /// 2. Check account existence and permissions
    /// 3. Execute the appropriate operation based on transaction type
    /// 4. Update global state with the results
    /// 5. Return success/failure status
    /// 
    /// ATOMICITY: All state changes within a transaction are atomic - either
    /// all succeed or all fail. This ensures data consistency even if the
    /// system crashes during transaction processing.
    /// 
    /// ERROR HANDLING: Uses catch_unwind to prevent panics from crashing the entire
    /// system. This is crucial in blockchain systems where one bad transaction
    /// shouldn't affect others.
    /// 
    /// GAS ACCOUNTING: In real blockchains, each operation costs gas, and
    /// transactions have gas limits. This implementation is simplified and
    /// doesn't include gas accounting.
    /// 
    /// RETURN VALUE: Returns a Result indicating success/failure and any error codes
    pub fn run_tx(&mut self, tx: Transaction) -> Result {
        match tx.tx_type {
            TransactionType::Transfer => {
                // EDUCATIONAL: Value transfers between accounts
                // This would involve updating account balances and checking sufficient funds
                panic!("regular call not implemented");
            }

            TransactionType::CreateAccount => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    self.create_account(tx.from, tx.to, tx.data);
                }));

                // EDUCATIONAL: Handle deployment failures gracefully
                if let Err(_e) = result {
                    return Result { error_code: 0, success: false };
                } else {
                    return Result { error_code: 1, success: true };
                }
            }

            TransactionType::ProgramCall => {
                // EDUCATIONAL: Execute an existing smart contract
                // First verify the destination is actually a contract
                assert!(self.state.is_contract(tx.to), "destination address is not a contract");

                // EDUCATIONAL: Call the contract and extract the result
                let (result_ptr, context_index) = self.call_contract(tx.from, tx.to, tx.data);

                // verify context stack is empty
                if !self.context_stack.is_empty() {
                    if self.context_stack.iter().any(|ctx| !ctx.exe_done) {
                        panic!("context stack has unfinished contexts after tx execution");
                    }
                }

                // extract result 
                self.extract_result(result_ptr, context_index)
            }
        }
    }

    /// Extracts the result of a contract execution from memory.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how contract results are communicated
    /// back to the caller. The contract writes its result to a specific memory location,
    /// and this function reads it.
    /// 
    /// RESULT FORMAT: The result is stored as a 5-byte structure:
    /// - 4 bytes: error code (u32)
    /// - 1 byte: success flag (0 = false, non-zero = true)
    /// 
    /// MEMORY SAFETY: Validates that the result pointer is within bounds
    /// to prevent reading invalid memory.
    fn extract_result(&self, result_ptr: u32, context_index: usize) -> Result {
        // EDUCATIONAL: Get the memory page where the result was stored
        let ee = self.context_stack.get(context_index).expect("missing execution context");
        let vm = ee.vm.borrow();
        let page = vm.memory.borrow();
        // let page_rc = self.memory_manager.first_page().expect("No memory page allocated");

        // EDUCATIONAL: Borrow the memory page to read from it
        // let page = page_rc.borrow();
        let mem = page.mem();
        let start = result_ptr as usize;

        // EDUCATIONAL: Validate memory bounds to prevent out-of-bounds access
        if start + 5 > mem.len() {
            panic!("Result struct out of bounds at 0x{:08x}", start);
        }

        // EDUCATIONAL: Extract the result fields from memory
        let error_code = u32::from_le_bytes(mem[start+1..start + 5].try_into().unwrap());
        let success = mem[start] != 0;

        return Result { error_code, success };
    }

    /// Creates a new account (smart contract) with the provided code.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates smart contract deployment.
    /// In blockchain systems, deploying a contract creates a new account
    /// that can hold code and persistent storage.
    /// 
    /// SECURITY CHECKS:
    /// - Ensures the target address isn't already in use
    /// - Validates code size limits to prevent resource exhaustion
    /// 
    /// ACCOUNT CREATION: Creates an Account struct with:
    /// - code: The smart contract bytecode
    /// - storage: Empty persistent storage
    /// - balance: 0 (no initial funds)
    /// - nonce: 0 (no transactions yet)
    /// - is_contract: true (marks this as a contract account)
   pub fn create_account(&mut self, _from: Address, to: Address, data: Vec<u8>) {
        // EDUCATIONAL: Deploy a new smart contract
        // This creates a new account with the provided code
        let is_contract = !data.is_empty();
        let code_size = data.len();

        println!(
            "Tx creating account at address {}. Is contract: {}. Code size: {} bytes.",
            to,
            is_contract,
            code_size
        );
    
        // EDUCATIONAL: Check that the target address is not already in use
        // This prevents overwriting existing accounts
        if self.state.accounts.contains_key(&to) {
            panic!("account already exists");
        }

        // EDUCATIONAL: Validate code size limits
        // This prevents resource exhaustion attacks
        let max = Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT;
        if data.len() > max {
            panic!(
                "❌ Code size ({}) exceeds CODE_SIZE_LIMIT ({} bytes)",
                data.len(),
                max
            );
        }

        // EDUCATIONAL: Create and insert new account with code
        let account = Account {
            code: data,                    // The smart contract bytecode
            storage: Default::default(),   // Empty persistent storage
            balance: 0,                    // No initial balance
            nonce: 0,                      // No transactions yet
            is_contract: true,             // Mark as contract account
        };

        self.state.accounts.insert(to, account);
    }

    /// Handles calling a new contract, spinning up a fresh VM with its own memory page.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates smart contract execution.
    /// Each contract call gets its own isolated VM instance to prevent
    /// interference between contracts.
    /// 
    /// EXECUTION PROCESS:
    /// 1. Validate the target is a contract
    /// 2. Allocate fresh memory and storage
    /// 3. Set up the VM with contract code and parameters
    /// 4. Execute the contract safely
    /// 5. Extract and return the result
    /// 
    /// ISOLATION: Each contract gets its own memory page and storage
    /// to prevent one contract from affecting another.
    /// 
    /// PARAMETER PASSING: Contract parameters are passed through CPU registers:
    /// - a0: Contract address (who is being called)
    /// - a1: Caller address (who is making the call)
    /// - a2: Input data pointer
    /// - a3: Input data length
    /// - a4: Result pointer (where to write the result)
    pub fn call_contract(&mut self, from: Address, to: Address, input_data: Vec<u8>) -> (u32, usize) {
        println!(
            "Tx calling program at address {} with data 0x{}",
            to,
            hex::encode(&input_data)
        );

        // SAFETY NOTE:
        // This line creates a HostShim containing a raw pointer (*mut AVM) to self.
        // Even though raw pointers don't participate in Rust's borrow checker,
        // calling `HostShim::new(self)` still *temporarily borrows* `self` as `&mut AVM`
        // during this line. If `self` is already mutably borrowed (e.g. for pushing to context_stack,
        // accessing state, or memory_manager), this will cause a compile-time error due to overlapping mutable borrows.
        // To avoid this, ensure all other mutable uses of `self` happen *before* or *after* this line.
        let shim = HostShim::new(self);

        // EDUCATIONAL: Get mutable reference to the contract account
        let account = self.state.get_account_mut(&to);
        if !account.is_contract {
            panic!("destination address {} is not a contract", to);
        }
        
        // EDUCATIONAL: Allocate memory and clone storage for isolation
        let memory_page = self.memory_manager.new_page();
        let storage = Rc::new(RefCell::new(Storage::with_map(account.storage.clone())));

        // EDUCATIONAL: Create and configure child VM
        // We use Box here to heap-allocate the HostShim and pass it as a trait object (Box<dyn HostInterface>).
        // This is necessary because `VM` stores the host as `Box<dyn HostInterface>`, which:
        // - Allows us to erase the concrete type (HostShim) at compile time
        // - Removes the need for lifetimes like &'a mut dyn HostInterface
        // - Enables recursive call_contract logic, since the Box owns the host and doesn't borrow `self`
        // Without Box, we would need to track lifetimes manually and would hit borrow checker issues.
        let mut vm: VM = VM::new(memory_page, storage.clone(), Box::new(shim));
        vm.set_code(Config::PROGRAM_START_ADDR, &account.code);
        // vm.cpu.verbose = true;

        // add new context execution
        let context_index = self.context_stack.push(from, to, input_data, vm);
        let context = self.context_stack.current_mut().expect("missing execution context");

        // EDUCATIONAL: Set up function parameters in registers
        // This follows the RISC-V calling convention
        let _address_ptr = context.vm.borrow_mut().set_reg_to_data(Register::A0, to.0.as_ref());      // Contract address
        let _pubkey_ptr = context.vm.borrow_mut().set_reg_to_data(Register::A1, from.0.as_ref());     // Caller address

        // EDUCATIONAL: Validate input size to prevent resource exhaustion
        let input_len = context.input_data.len();
        if input_len > Config::MAX_INPUT_LEN {
            panic!(
                "Entrypoint: input length {} exceeds MAX_INPUT_LEN ({})",
                input_len,
                Config::MAX_INPUT_LEN
            );
        }

        // EDUCATIONAL: Set up input data and result pointer
        let _input_ptr = context.vm.borrow_mut().set_reg_to_data(Register::A2, &context.input_data);          // Input data
        context.vm.borrow_mut().set_reg_u32(Register::A3, input_len as u32);                          // Input length
        let result_ptr = context.vm.borrow_mut().set_reg_to_data(Register::A4, &[0u8; 5]);           // Result buffer

        // EDUCATIONAL: Run the VM safely with panic handling
        let result = catch_unwind(AssertUnwindSafe(|| {
            context.vm.borrow_mut().raw_run();
        }));

        // EDUCATIONAL: Handle VM panics gracefully
        if let Err(e) = result {
            eprintln!("💥 VM panicked: {:?}", e);
            panic!("VM panicked");
        }

        // EDUCATIONAL: Copy storage back into account
        // This persists any changes the contract made to storage
        let updated_map = storage.borrow().map.borrow().clone();
        account.storage = updated_map;

        // EDUCATIONAL: set context execution done
        context.exe_done = true;

        (result_ptr, context_index)
    }

    /// Peek the current active execution context.
    /// 
    /// EDUCATIONAL PURPOSE: This allows inspection of the current execution
    /// context, which is useful for debugging and understanding the call stack.
    /// 
    /// USAGE: Typically used by debugging tools or for implementing features
    /// like gas accounting or call tracing.
    pub fn current_context(&self) -> Option<&ExecutionContext> {
        self.context_stack.current()
    }
}
