use std::cell::RefCell;
use std::rc::Rc;

use avm::transaction::{TransactionBundle, TransactionType};
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::Elf;

use crate::memory::StackedMemory;
use crate::traps::{TrapAction, TrapFrame, TrapHandler, TrapTable};
use storage::Storage;
use vm::host_interface::NoopHost;
use vm::memory::{Memory, HEAP_PTR_OFFSET};
use vm::metering::{MemoryAccessKind, NoopMeter};
use vm::registers::Register;
use vm::vm::VM;

/// Boot configuration options consumed by the loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootConfig {
    pub debug_console: bool,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self { debug_console: true }
    }
}

/// Bootloader skeleton that loads a kernel image into fresh memory and
/// mediates syscall traps until the kernel installs its own handlers.
#[derive(Debug)]
pub struct Bootloader {
    pub config: BootConfig,
    memory: StackedMemory,
    traps: TrapTable,
}

impl Bootloader {
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        Self {
            config: BootConfig::default(),
            memory: StackedMemory::new(max_pages, page_size),
            traps: TrapTable::new(),
        }
    }

    /// Register a syscall trap handler that will run before transferring control to the kernel.
    pub fn register_syscall_trap(&mut self, syscall_id: u32, handler: TrapHandler) {
        self.traps.register(syscall_id, handler);
    }

    /// Load an ELF kernel image into a fresh page and return its entry point + backing memory.
    pub fn load_kernel(&mut self, elf_bytes: &[u8]) -> (u32, Memory) {
        let elf = Elf::parse(elf_bytes).expect("failed to parse kernel ELF");
        let entry_point = elf.entry as u32;
        let page = self.memory.new_page();
        let mut meter = NoopMeter::default();
        let mut highest_byte = 0usize;

        for header in elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == PT_LOAD && ph.p_filesz > 0)
        {
            let offset = header.p_offset as usize;
            let file_size = header.p_filesz as usize;
            let start = header.p_vaddr as usize;
            let end = start
                .checked_add(file_size)
                .expect("segment size overflow");

            assert!(
                end <= page.size(),
                "ELF segment does not fit in a single page (need {}, have {})",
                end,
                page.size()
            );

            let segment = &elf_bytes[offset..offset + file_size];
            for (idx, byte) in segment.iter().enumerate() {
                if !page.store_u8(
                    start + idx,
                    *byte,
                    &mut meter,
                    MemoryAccessKind::Store,
                ) {
                    panic!("failed to write kernel segment to memory");
                }
            }

            highest_byte = highest_byte.max(end);
        }

        // Align heap pointer after loaded image.
        let heap_start = highest_byte + HEAP_PTR_OFFSET as usize;
        page.set_next_heap(heap_start as u32);

        (entry_point, page)
    }

    /// Dispatch a syscall trap using the boot-time trap table.
    pub fn handle_trap(&self, frame: &mut TrapFrame) -> TrapAction {
        self.traps.dispatch(frame)
    }

    /// Execute a transaction bundle by delegating to the kernel. This mirrors the
    /// AVM entry point where the kernel is responsible for invoking programs.
    pub fn execute_bundle(&mut self, kernel_elf: &[u8], bundle: &TransactionBundle) {
        let (entry_point, memory) = self.load_kernel(kernel_elf);
        let storage = Rc::new(RefCell::new(Storage::new()));
        let host: Box<dyn vm::host_interface::HostInterface> = Box::new(NoopHost);

        let mut vm = VM::new(memory.clone(), storage, host);
        self.place_bundle(&mut vm, bundle);
        vm.cpu.pc = entry_point;

        // TODO: write the bundle into memory for the kernel to consume, and
        // extend VM host to forward syscalls into the trap table.
        vm.raw_run();
    }

    fn place_bundle(&mut self, vm: &mut VM, bundle: &TransactionBundle) {
        let encoded = encode_bundle(bundle);
        let addr = vm.set_reg_to_data(Register::A0, &encoded);
        // Register a length hint so the kernel can bounds-check the payload.
        vm.set_reg_u32(Register::A1, encoded.len() as u32);
        // Keep heap aligned after our write.
        vm.memory
            .set_next_heap((addr as usize + encoded.len() + HEAP_PTR_OFFSET as usize) as u32);
    }
}

fn encode_bundle(bundle: &TransactionBundle) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(bundle.transactions.len() as u32).to_le_bytes());

    for tx in &bundle.transactions {
        let tx_type = match tx.tx_type {
            TransactionType::Transfer => 0,
            TransactionType::CreateAccount => 1,
            TransactionType::ProgramCall => 2,
        };
        out.push(tx_type);
        out.extend_from_slice(&tx.to.0);
        out.extend_from_slice(&tx.from.0);
        out.extend_from_slice(&(tx.data.len() as u32).to_le_bytes());
        out.extend_from_slice(&tx.data);
        out.extend_from_slice(&tx.value.to_le_bytes());
        out.extend_from_slice(&tx.nonce.to_le_bytes());
    }

    out
}
