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

    pub events: Vec<Vec<u8>>, 

    // is exe_done marks context as executed
    pub exe_done: bool,
}

impl ExecutionContext {
    pub fn new(
        from: Address,
         to: Address,
         input_data: Vec<u8>,
         vm: VM,
    ) -> Self {
        Self { 
            from,
            to,
            input_data: Rc::new(input_data), 
            vm: Rc::new(RefCell::new(vm)), 
            events: Vec::new(),
            exe_done: false,
         }
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
    /// returns index of the new execution context
    pub fn push(&mut self, from: Address, to: Address, input_data: Vec<u8>, vm: VM) -> usize {
        let index = self.stack.len();
        self.stack.push(
            ExecutionContext { 
                from, 
                to, 
                input_data:Rc::new(input_data), 
                vm:Rc::new(RefCell::new(vm)), 
                events: Vec::new(),
                exe_done: false,
            });
        index
    }

    /// Pop the most recent context off the stack (e.g., when returning from a call).
    pub fn pop(&mut self) -> Option<ExecutionContext> {
        self.stack.pop()
    }

    /// Peek execution context index without modifying the stack.
    pub fn get(&self, i: usize) -> Option<&ExecutionContext> {
        self.stack.get(i)
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut ExecutionContext> {
        self.stack.get_mut(i)
    }

    /// Peek at the current execution context without modifying the stack.
    pub fn current(&self) -> Option<&ExecutionContext> {
        self.stack.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut ExecutionContext> {
        self.stack.last_mut()
    }


    pub fn iter(&self) -> impl Iterator<Item = &ExecutionContext> {
        self.stack.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Collect all events from a starting context index through the top of the stack.
    pub fn collect_events_from(&self, start: usize) -> Vec<Vec<u8>> {
        self.stack
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx >= start)
            .flat_map(|(_, ctx)| ctx.events.clone())
            .collect()
    }
}
