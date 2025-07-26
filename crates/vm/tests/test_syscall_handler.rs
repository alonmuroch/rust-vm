use std::rc::Rc;
use core::cell::RefCell;
use vm::memory_page::MemoryPage;
use storage::Storage;
use vm::host_interface::HostInterface;
use vm::sys_call::SyscallHandler;
use vm::registers::Register;
use std::any::Any;

#[derive(Debug)]
pub struct TestSyscallHandler {
    tohost_addr: u64,
    memory: Option<Rc<RefCell<MemoryPage>>>,
}

impl TestSyscallHandler {
    pub fn new() -> Self {
        Self { tohost_addr: 0, memory: None }
    }

    /// Set the address of the .tohost section
    pub fn set_tohost_addr(&mut self, addr: u64) {
        self.tohost_addr = addr;
    }

    /// Set the memory reference (needed to read .tohost)
    pub fn set_memory(&mut self, memory: Rc<RefCell<MemoryPage>>) {
        self.memory = Some(memory);
    }
}

pub const SYSCALL_TEST_DONE: u32 = 0;
pub const SYSCALL_TERMINATE: u32 = 93;

impl SyscallHandler for TestSyscallHandler {
    fn handle_syscall(
        &mut self,
        call_id: u32,
        _args: [u32; 6],
        memory: Rc<RefCell<MemoryPage>>,
        _storage: Rc<RefCell<Storage>>,
        _host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
    ) -> (u32, bool) {
        let mut result = 0;
        match call_id {
            SYSCALL_TEST_DONE => {
                // Read .tohost value
                let mem_ref = self.memory.as_ref().unwrap_or(&memory);
                let offset = mem_ref.borrow().offset(self.tohost_addr as usize);
                let mem_guard = mem_ref.borrow();
                let mem = mem_guard.mem();
                if offset + 8 <= mem.len() {
                    let tohost_val = u64::from_le_bytes(mem[offset..offset+8].try_into().unwrap());
                    // Use .tohost value as the test result
                    result = tohost_val as u32;
                } else {
                    panic!("[TestSyscallHandler] .tohost address out of bounds");
                }
                if result == 0 {
                    return (result, true);
                } 
                panic!("[spec-test] FAIL: .tohost value = 0x{:x}", result);
            },
            SYSCALL_TERMINATE => {
                let exit_code = regs[Register::A0 as usize];
                return (exit_code, false); // halt VM
            },
            _ => {
                panic!("Unknown syscall ID: {}", call_id);
            }
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
} 