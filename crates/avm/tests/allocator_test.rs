use avm::memory::MemoryPage;
use std::cell::RefCell;
use std::rc::Rc;
use storage::Storage;
use vm::host_interface;
use vm::metering::NoopMeter;
use vm::memory::SharedMemory;
use vm::sys_call::{DefaultSyscallHandler, SyscallHandler, SYSCALL_ALLOC, SYSCALL_DEALLOC};

#[test]
fn test_allocator_syscalls() {
    let memory: SharedMemory = Rc::new(MemoryPage::new(8192));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut meter = NoopMeter::default();

    // Test SYSCALL_ALLOC
    let args = [1024, 8, 0, 0, 0, 0];
    let mut regs = [0u32; 32];
    let (result, _) = syscall_handler.handle_syscall(
        SYSCALL_ALLOC,
        args,
        memory.clone(),
        storage.clone(),
        &mut host,
        &mut regs,
        &mut meter,
    );

    assert_ne!(result, 0);

    // Test SYSCALL_DEALLOC (no-op but should not crash)
    let dealloc_args = [result, 1024, 0, 0, 0, 0];
    let (dealloc_result, _) = syscall_handler.handle_syscall(
        SYSCALL_DEALLOC,
        dealloc_args,
        memory.clone(),
        storage.clone(),
        &mut host,
        &mut regs,
        &mut meter,
    );

    assert_eq!(dealloc_result, 0);
}

#[test]
fn test_multiple_allocations() {
    let memory: SharedMemory = Rc::new(MemoryPage::new(8192));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut regs = [0u32; 32];
    let mut meter = NoopMeter::default();

    let mut pointers = Vec::new();

    // Allocate multiple blocks
    for i in 0..5 {
        let size = 64 + i * 32;
        let args = [size, 4, 0, 0, 0, 0];

        let (ptr, _) = syscall_handler.handle_syscall(
            SYSCALL_ALLOC,
            args,
            memory.clone(),
            storage.clone(),
            &mut host,
            &mut regs,
            &mut meter,
        );

        assert_ne!(ptr, 0);
        pointers.push(ptr);
    }

    // Verify pointers are aligned
    for &ptr in &pointers {
        assert_eq!(ptr % 4, 0);
    }

    // Verify no overlapping pointers (simple check)
    for i in 0..pointers.len() {
        for j in i + 1..pointers.len() {
            assert_ne!(pointers[i], pointers[j]);
        }
    }
}

#[test]
fn test_alignment_requirements() {
    let memory: SharedMemory = Rc::new(MemoryPage::new(8192));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut regs = [0u32; 32];
    let mut meter = NoopMeter::default();

    // Test various alignments
    let alignments = [1, 2, 4, 8, 16];

    for &align in &alignments {
        let args = [256, align as u32, 0, 0, 0, 0];
        let (ptr, _) = syscall_handler.handle_syscall(
            SYSCALL_ALLOC,
            args,
            memory.clone(),
            storage.clone(),
            &mut host,
            &mut regs,
            &mut meter,
        );

        assert_ne!(ptr, 0);
        assert_eq!(ptr as usize % align, 0);
    }
}

#[test]
fn test_invalid_alignment() {
    let memory: SharedMemory = Rc::new(MemoryPage::new(8192));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut regs = [0u32; 32];
    let mut meter = NoopMeter::default();

    // Test invalid alignments (not powers of 2)
    let invalid_alignments = [0, 3, 5, 6, 7, 9];

    for &align in &invalid_alignments {
        let args = [100, align as u32, 0, 0, 0, 0];
        let (ptr, _) = syscall_handler.handle_syscall(
            SYSCALL_ALLOC,
            args,
            memory.clone(),
            storage.clone(),
            &mut host,
            &mut regs,
            &mut meter,
        );

        assert_eq!(ptr, 0);
    }
}
