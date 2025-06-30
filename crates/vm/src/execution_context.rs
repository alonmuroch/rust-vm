/// Represents a 20-byte address (e.g., Ethereum-style account or contract).
pub type Address = [u8; 20];

/// Represents a single execution context during contract calls.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The address that initiated the current call.
    pub from: Address,

    /// The address currently receiving the call.
    pub to: Address,
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
    pub fn push(&mut self, from: Address, to: Address) {
        self.stack.push(ExecutionContext { from, to });
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
