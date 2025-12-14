use std::collections::HashMap;
use std::fmt;

/// Outcome after a trap handler runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapAction {
    Continue,
    KillKernel,
}

/// Lightweight snapshot of CPU state for a syscall trap.
#[derive(Debug, Clone)]
pub struct TrapFrame {
    pub syscall_id: u32,
    pub args: [u32; 6],
    pub pc: usize,
}

pub type TrapHandler = Box<dyn Fn(&mut TrapFrame) -> TrapAction + Send + Sync>;

/// Syscall IDs mirrored from the VM syscall table.
pub mod syscall {
    pub const STORAGE_GET: u32 = 1;
    pub const STORAGE_SET: u32 = 2;
    pub const PANIC: u32 = 3;
    pub const LOG: u32 = 4;
    pub const CALL_PROGRAM: u32 = 5;
    pub const FIRE_EVENT: u32 = 6;
    pub const ALLOC: u32 = 7;
    pub const DEALLOC: u32 = 8;
    pub const TRANSFER: u32 = 9;
    pub const BALANCE: u32 = 10;
}

/// Simple table that dispatches traps by syscall id.
#[derive(Default)]
pub struct TrapTable {
    handlers: HashMap<u32, TrapHandler>,
}

impl fmt::Debug for TrapTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrapTable")
            .field("registered", &self.handlers.len())
            .finish()
    }
}

impl TrapTable {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, syscall_id: u32, handler: TrapHandler) {
        self.handlers.insert(syscall_id, handler);
    }

    pub fn dispatch(&self, frame: &mut TrapFrame) -> TrapAction {
        if let Some(handler) = self.handlers.get(&frame.syscall_id) {
            handler(frame)
        } else {
            TrapAction::KillKernel
        }
    }
}
