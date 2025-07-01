use vm::{Address, ExecutionContext};

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
