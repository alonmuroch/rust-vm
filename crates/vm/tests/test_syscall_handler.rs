use std::rc::Rc;
use core::cell::RefCell;
use vm::memory_page::MemoryPage;
use storage::Storage;
use vm::host_interface::HostInterface;
use vm::cpu::CPU;
use vm::sys_call::{SyscallHandler, SYSCALL_SPEC_TEST};
use vm::registers::Register;

#[derive(Debug)]
pub struct TestSyscallHandler;

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
        if syscall_id == SYSCALL_SPEC_TEST {
            let result = regs[Register::A0 as usize];
            if result == 0 {
                println!("[spec-test] PASS: SYSCALL_SPEC_TEST called: a0 (result) = 0");
            } else {
                println!("[spec-test] FAIL: SYSCALL_SPEC_TEST called: a0 (result) = {}", result);
            }
            return (0, true);
        }
        println!("[TestSyscallHandler] handle_syscall called (id = {})", syscall_id);
        (0, true)
    }
} 