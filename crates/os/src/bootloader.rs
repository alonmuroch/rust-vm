use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use compiler::elf::parse_elf_from_bytes;
use goblin::elf::Elf;
use types::transaction::TransactionBundle;

use crate::DefaultSyscallHandler;
use crate::memory::StackedMemory;
use state::State;
use vm::host_interface::NoopHost;
use vm::memory::{HEAP_PTR_OFFSET, Memory};
use vm::registers::Register;
use vm::vm::VM;

/// Boot configuration options consumed by the loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootConfig {
    pub debug_console: bool,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            debug_console: true,
        }
    }
}

/// Bootloader skeleton that loads a kernel image into fresh memory and
/// hands control to the kernel.
#[derive(Debug)]
pub struct Bootloader {
    pub config: BootConfig,
    memory: StackedMemory,
}

impl Bootloader {
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        Self {
            config: BootConfig::default(),
            memory: StackedMemory::new(max_pages, page_size),
        }
    }

    /// Load an ELF kernel image into a fresh page and return its entry point + backing memory.
    pub fn load_kernel(&mut self, elf_bytes: &[u8]) -> (u32, Memory) {
        let elf = parse_elf_from_bytes(elf_bytes).expect("failed to parse kernel ELF");
        let entry_point = Elf::parse(elf_bytes)
            .expect("failed to parse entry point")
            .entry as u32;

        let (code, code_base) = elf.get_flat_code().expect("kernel ELF missing .text");
        let (rodata, ro_base) = elf.get_flat_rodata().unwrap_or((Vec::new(), code_base));

        let min_base = core::cmp::min(code_base, ro_base) as usize;
        let code_end = (code_base + code.len() as u64) as usize;
        let ro_end = (ro_base + rodata.len() as u64) as usize;
        let image_end = core::cmp::max(code_end, ro_end);
        let image_size = image_end.checked_sub(min_base).expect("invalid image size");

        let page = self.memory.new_page();
        assert!(
            image_end <= page.size(),
            "ELF image does not fit in a single page (need {}, have {})",
            image_end,
            page.size()
        );

        // Flatten code + rodata into a single buffer and write once to set heap pointer properly.
        let mut image = vec![0u8; image_size];
        let code_off = (code_base as usize).saturating_sub(min_base);
        image[code_off..code_off + code.len()].copy_from_slice(&code);
        if !rodata.is_empty() {
            let ro_off = (ro_base as usize).saturating_sub(min_base);
            image[ro_off..ro_off + rodata.len()].copy_from_slice(&rodata);
        }

        page.write_code(min_base, &image);
        (entry_point, page)
    }

    /// Execute a transaction bundle by delegating to the kernel. This mirrors the
    /// AVM entry point where the kernel is responsible for invoking programs.
    pub fn execute_bundle(&mut self, kernel_elf: &[u8], bundle: &TransactionBundle) {
        let (entry_point, memory) = self.load_kernel(kernel_elf);
        let state = Rc::new(RefCell::new(State::new()));
        let host: Box<dyn vm::host_interface::HostInterface> = Box::new(NoopHost);

        let mut vm = VM::new(memory.clone(), host, Box::new(DefaultSyscallHandler::new(state)));
        self.place_bundle(&mut vm, bundle);
        vm.cpu.pc = entry_point;

        // TODO: write the bundle into memory for the kernel to consume.
        vm.raw_run();
    }

    fn place_bundle(&mut self, vm: &mut VM, bundle: &TransactionBundle) {
        let encoded = bundle.encode();
        let addr = vm.set_reg_to_data(Register::A0, &encoded);
        // Register a length hint so the kernel can bounds-check the payload.
        vm.set_reg_u32(Register::A1, encoded.len() as u32);
        // Keep heap aligned after our write.
        vm.memory
            .set_next_heap((addr as usize + encoded.len() + HEAP_PTR_OFFSET as usize) as u32);
    }
}
