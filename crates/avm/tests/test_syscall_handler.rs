use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use storage::Storage;
use vm::host_interface::HostInterface;
use vm::metering::Metering;
use vm::memory::SharedMemory;
use vm::registers::Register;
use vm::sys_call::SyscallHandler;

/// Map RISC-V test exit codes to test case numbers
/// Formula: exit_code = (TESTNUM << 1) | 1
/// So: TESTNUM = (exit_code - 1) >> 1
fn exit_code_to_test_num(exit_code: u32) -> Option<u32> {
    if exit_code == 0 {
        None // 0 means test passed
    } else if exit_code % 2 == 1 {
        Some((exit_code - 1) >> 1)
    } else {
        None // Even exit codes are not from RVTEST_FAIL
    }
}

#[derive(Debug)]
pub struct TestSyscallHandler {
    tohost_addr: u64,
    memory: Option<SharedMemory>,
}

impl TestSyscallHandler {
    pub fn new() -> Self {
        Self {
            tohost_addr: 0,
            memory: None,
        }
    }

    /// Set the address of the .tohost section
    pub fn set_tohost_addr(&mut self, addr: u64) {
        self.tohost_addr = addr;
    }

    /// Set the memory reference (needed to read .tohost)
    pub fn set_memory(&mut self, memory: SharedMemory) {
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
        memory: SharedMemory,
        _storage: Rc<RefCell<Storage>>,
        _host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
        _metering: &mut dyn Metering,
    ) -> (u32, bool) {
        let mut result = 0;
        match call_id {
            SYSCALL_TEST_DONE => {
                // Read .tohost value
                let mem_ref = self.memory.as_ref().unwrap_or(&memory);
                let offset = mem_ref.offset(self.tohost_addr as usize);
                let mem = mem_ref.mem();
                if offset + 8 <= mem.len() {
                    let tohost_val =
                        u64::from_le_bytes(mem[offset..offset + 8].try_into().unwrap());
                    // Use .tohost value as the test result
                    result = tohost_val as u32;
                } else {
                    panic!("[TestSyscallHandler] .tohost address out of bounds");
                }
                if result == 0 {
                    return (result, true);
                }
                panic!("[spec-test] FAIL: .tohost value = 0x{:x}", result);
            }
            SYSCALL_TERMINATE => {
                let exit_code = regs[Register::A0 as usize];
                if exit_code != 0 {
                    // Try to map exit code to test case number
                    if let Some(test_num) = exit_code_to_test_num(exit_code) {
                        panic!(
                            "[spec-test] FAIL: Test case {} failed (exit code {})",
                            test_num, exit_code
                        );
                    } else {
                        panic!("[spec-test] FAIL: Test failed with exit code {}", exit_code);
                    }
                }
                return (exit_code, false); // halt VM
            }
            _ => {
                panic!("Unknown syscall ID: {}", call_id);
            }
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
