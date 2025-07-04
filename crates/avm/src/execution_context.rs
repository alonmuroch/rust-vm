use types::address::Address;
use std::rc::Rc;
use std::cell::RefCell;
use vm::vm::VM;

/// Represents a single execution context during contract calls.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The address that initiated the current call.
    pub from: Address,

    /// The address currently receiving the call.
    pub to: Address,

    // Data passed to the contract call
    pub input_data: Rc<Vec<u8>>, 

    // Memory page
    pub vm: Rc<RefCell<VM>>,
}

impl ExecutionContext {
    pub fn new(
        from: Address,
         to: Address,
         input_data: Vec<u8>,
         vm: VM,
    ) -> Self {
        Self { from, to, input_data: Rc::new(input_data), vm: Rc::new(RefCell::new(vm)) }
    }
}

/// A call stack for nested execution contexts in the VM.
#[derive(Debug)]
pub struct ContextStack {
    stack: Vec<ExecutionContext>,
}

impl ContextStack {
    /// Create a new, empty context stack.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push a new context onto the stack (e.g., when a contract calls another).
    pub fn push(&mut self, from: Address, to: Address, input_data: Vec<u8>, vm: VM) {
        self.stack.push(ExecutionContext { from, to, input_data:Rc::new(input_data), vm:Rc::new(RefCell::new(vm)) });
    }

    /// Pop the most recent context off the stack (e.g., when returning from a call).
    pub fn pop(&mut self) -> Option<ExecutionContext> {
        self.stack.pop()
    }

    /// Peek at the current execution context without modifying the stack.
    pub fn current(&self) -> Option<&ExecutionContext> {
        self.stack.last()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}
