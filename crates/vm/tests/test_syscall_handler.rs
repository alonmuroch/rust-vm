use std::rc::Rc;
use core::cell::RefCell;
use vm::memory_page::MemoryPage;
use storage::Storage;
use vm::host_interface::HostInterface;
use vm::sys_call::SyscallHandler;
use vm::registers::Register;

pub const SYSCALL_SPEC_TEST: u32 = 0;

#[derive(Debug)]
pub struct TestSyscallHandler {
    calls: Rc<RefCell<Vec<(u32, u32)>>>, // (syscall_id, result)
}

impl TestSyscallHandler {
    pub fn new() -> Self {
        Self { calls: Rc::new(RefCell::new(Vec::new())) }
    }
}

impl Drop for TestSyscallHandler {
    fn drop(&mut self) {
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
            panic!("###### One or more syscalls returned non-zero result! ######");
        } else {
            println!("###### All syscalls returned result = 0 ######");
        }
    }
}

impl SyscallHandler for TestSyscallHandler {
    fn handle_syscall(
        &mut self,
        args: [u32; 6],
        _memory: Rc<RefCell<MemoryPage>>,
        _storage: Rc<RefCell<Storage>>,
        _host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
    ) -> (u32, bool) {
        let syscall_id = regs[Register::A7 as usize];
        let mut result = 0;
        if syscall_id == SYSCALL_SPEC_TEST {
            result = regs[Register::A0 as usize];
            if result == 0 {
                println!("[spec-test] PASS: SYSCALL_SPEC_TEST called: a0 (result) = 0");
            } else {
                println!("[spec-test] FAIL: SYSCALL_SPEC_TEST called: a0 (result) = {}", result);
            }
        } else {
            println!("[TestSyscallHandler] handle_syscall called (id = {})", syscall_id);
        }
        self.calls.borrow_mut().push((syscall_id, result));
        (result, true)
    }
} 