use std::rc::Rc;
use core::cell::RefCell;
use vm::memory_page::MemoryPage;
use storage::Storage;
use vm::host_interface::HostInterface;
use vm::sys_call::SyscallHandler;
use vm::registers::Register;
use std::any::Any;

pub const SYSCALL_SPEC_TEST: u32 = 0;

#[derive(Debug)]
pub struct TestSyscallHandler {
    pub calls: Rc<RefCell<Vec<(u32, u32)>>>, // (syscall_id, result)
    tohost_addr: Option<u64>,
    memory: Option<Rc<RefCell<MemoryPage>>>,
}

impl TestSyscallHandler {
    pub fn new() -> Self {
        Self { calls: Rc::new(RefCell::new(Vec::new())), tohost_addr: None, memory: None }
    }

    /// Set the address of the .tohost section
    pub fn set_tohost_addr(&mut self, addr: u64) {
        self.tohost_addr = Some(addr);
    }

    /// Set the memory reference (needed to read .tohost)
    pub fn set_memory(&mut self, memory: Rc<RefCell<MemoryPage>>) {
        self.memory = Some(memory);
    }

    /// Print the test results
    pub fn print_results(&self) {
        let calls = self.calls.borrow();
        let mut failed = false;
        println!("[TestSyscallHandler] Syscall results:");
        for (i, (_syscall_id, result)) in calls.iter().enumerate() {
             if *result != 0 {
                failed = true;
                println!("  Test {} Failed! with result = {}", i, result);
             } else {
                println!("  Test {} Successful!", i);
             }
        }
        if failed {
            println!("###### One or more syscalls returned non-zero result! ######");
        } else {
            println!("###### All syscalls returned result = 0 ######");
        }
    }
}

impl SyscallHandler for TestSyscallHandler {
    fn handle_syscall(
        &mut self,
        _args: [u32; 6],
        memory: Rc<RefCell<MemoryPage>>,
        _storage: Rc<RefCell<Storage>>,
        _host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
    ) -> (u32, bool) {
        let syscall_id = regs[Register::A7 as usize];
        let mut result = 0;
        
        // Read .tohost value if address is known
        let tohost_addr = self.tohost_addr;
        let mem_ref = self.memory.as_ref().unwrap_or(&memory);
        if let Some(addr) = tohost_addr {
            let offset = mem_ref.borrow().offset(addr as usize);
            let mem_guard = mem_ref.borrow();
            let mem = mem_guard.mem();
            if offset + 8 <= mem.len() {
                let tohost_val = u64::from_le_bytes(mem[offset..offset+8].try_into().unwrap());
                println!("[TestSyscallHandler] .tohost value: 0x{:x}", tohost_val);
                // Use .tohost value as the test result
                result = tohost_val as u32;
            } else {
                println!("[TestSyscallHandler] .tohost address out of bounds");
            }
        }
        
        if syscall_id == SYSCALL_SPEC_TEST {
            if result == 0 {
                println!("[spec-test] PASS: .tohost value = 0");
            } else {
                println!("[spec-test] FAIL: .tohost value = 0x{:x}", result);
            }
        } else {
            println!("[TestSyscallHandler] handle_syscall called (id = {})", syscall_id);
        }
        self.calls.borrow_mut().push((syscall_id, result));

        (result, true)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
} 