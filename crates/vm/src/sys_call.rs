use crate::host_interface::HostInterface;
use crate::memory::Memory;
use crate::metering::Metering;
use core::any::Any;

/// System call IDs for the VM.
pub const SYSCALL_STORAGE_GET: u32 = 1;
pub const SYSCALL_STORAGE_SET: u32 = 2;
pub const SYSCALL_PANIC: u32 = 3;
pub const SYSCALL_LOG: u32 = 4;
pub const SYSCALL_CALL_PROGRAM: u32 = 5;
pub const SYSCALL_FIRE_EVENT: u32 = 6;
pub const SYSCALL_ALLOC: u32 = 7;
pub const SYSCALL_DEALLOC: u32 = 8;
pub const SYSCALL_TRANSFER: u32 = 9;
pub const SYSCALL_BALANCE: u32 = 10;
pub const SYSCALL_COMMIT_STATE: u32 = 11;
pub const SYSCALL_CREATE_ACCOUNT: u32 = 12;
// Linux/RISC-V memory management syscall numbers:
pub const SYSCALL_MMAP: u32 = 222;      // mmap(2): map pages with PROT/FLAGS
pub const SYSCALL_MUNMAP: u32 = 215;    // munmap(2): unmap a VA range
pub const SYSCALL_MPROTECT: u32 = 226;  // mprotect(2): change page protections
pub const SYSCALL_BRK: u32 = 214;       // brk(2): set program break (heap end)

/// Trait implemented by syscall handlers consumed by the VM.
pub trait SyscallHandler: std::fmt::Debug {
    fn handle_syscall(
        &mut self,
        call_id: u32,
        args: [u32; 6],
        memory: Memory,
        host: &mut Box<dyn HostInterface>,
        regs: &mut [u32; 32],
        metering: &mut dyn Metering,
    ) -> (u32, bool);
    fn as_any(&self) -> &dyn Any;
}
