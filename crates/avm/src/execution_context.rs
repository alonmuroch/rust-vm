use types::address::Address;

/// Represents a single execution context during contract calls.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The address that initiated the current call.
    pub from: Address,

    /// The address currently receiving the call.
    pub to: Address,

    // Data passed to the contract call
    pub input_data: Vec<u8>, 
}
impl ExecutionContext {
    pub fn new(from: Address, to: Address, input_data: Vec<u8>) -> Self {
        Self { from, to, input_data }
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
    pub fn push(&mut self, from: Address, to: Address, input_data: Vec<u8>) {
        self.stack.push(ExecutionContext { from, to, input_data });
    }

    /// Pop the most recent context off the stack (e.g., when returning from a call).
    pub fn pop(&mut self) -> Option<ExecutionContext> {
        self.stack.pop()
    }

    /// Peek at the current execution context without modifying the stack.
    pub fn current(&self) -> Option<&ExecutionContext> {
        self.stack.last()
    }
}
