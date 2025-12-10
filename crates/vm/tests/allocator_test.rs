use vm::sys_call::{SyscallHandler, DefaultSyscallHandler, SYSCALL_ALLOC, SYSCALL_DEALLOC};
use vm::{memory_page, host_interface};
use vm::metering::NoopMeter;
use storage::Storage;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_allocator_syscalls() {
    let memory = Rc::new(RefCell::new(memory_page::MemoryPage::new(8192)));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut meter = NoopMeter::default();

    // Test SYSCALL_ALLOC
    let args = [
        1024, // size
        8,    // alignment  
        0, 0, 0, 0
    ];
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

    println!("✅ SYSCALL_ALLOC returned pointer: 0x{:08x}", result);
    assert_ne!(result, 0); // Should return valid pointer

    // Test SYSCALL_DEALLOC (no-op but should not crash)
    let dealloc_args = [
        result,  // pointer to deallocate
        1024,    // size
        0, 0, 0, 0
    ];
    let (dealloc_result, _) = syscall_handler.handle_syscall(
        SYSCALL_DEALLOC,
        dealloc_args,
        memory.clone(),
        storage.clone(),
        &mut host,
        &mut regs,
        &mut meter,
    );

    println!("✅ SYSCALL_DEALLOC returned: {}", dealloc_result);
    assert_eq!(dealloc_result, 0); // Should return 0 (success)
}

#[test] 
fn test_multiple_allocations() {
    let memory = Rc::new(RefCell::new(memory_page::MemoryPage::new(8192)));
    let storage = Rc::new(RefCell::new(Storage::new()));
    let mut host: Box<dyn host_interface::HostInterface> = Box::new(host_interface::NoopHost);
    let mut syscall_handler = DefaultSyscallHandler::new();
    let mut regs = [0u32; 32];
    let mut meter = NoopMeter::default();

    let mut pointers = Vec::new();

    // Allocate multiple blocks
    for i in 0..5 {
        let size = 64 + i * 32; // Different sizes
        let args = [size, 4, 0, 0, 0, 0]; // 4-byte alignment
        
        let (ptr, _) = syscall_handler.handle_syscall(
            SYSCALL_ALLOC,
            args,
            memory.clone(),
            storage.clone(),
            &mut host,
            &mut regs,
            &mut meter,
        );
        
        println!("✅ Allocation {}: size={}, ptr=0x{:08x}", i, size, ptr);
        assert_ne!(ptr, 0);
        pointers.push(ptr);
    }

    // Verify pointers are different and properly aligned
    for (i, &ptr) in pointers.iter().enumerate() {
        assert!(ptr % 4 == 0, "Allocation {} not aligned: 0x{:08x}", i, ptr);
    }

    // Verify no overlapping pointers (simple check)
    for i in 0..pointers.len() {
        for j in i+1..pointers.len() {
            assert_ne!(pointers[i], pointers[j], "Duplicate pointers: 0x{:08x}", pointers[i]);
        }
    }
}

#[test]
fn test_alignment_requirements() {
    let memory = Rc::new(RefCell::new(memory_page::MemoryPage::new(8192)));
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
        
        println!("✅ Alignment test: align={}, ptr=0x{:08x}", align, ptr);
        assert_ne!(ptr, 0);
        assert!(ptr as usize % align == 0, 
               "Pointer 0x{:08x} not aligned to {} bytes", ptr, align);
    }
}

#[test]
fn test_invalid_alignment() {
    let memory = Rc::new(RefCell::new(memory_page::MemoryPage::new(8192)));
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
        
        println!("✅ Invalid alignment test: align={}, ptr=0x{:08x}", align, ptr);
        assert_eq!(ptr, 0, "Should return null for invalid alignment {}", align);
    }
}
