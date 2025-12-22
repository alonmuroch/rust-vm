use core::{mem, slice};
use core::fmt::Write as FmtWrite;
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use compiler::elf::parse_elf_from_bytes;
use goblin::elf::Elf;
use types::{boot::BootInfo, transaction::TransactionBundle};

use crate::DefaultSyscallHandler;
use crate::memory::{Memory, Perms};
use state::State;
use vm::host_interface::NoopHost;
use vm::memory::{Mmu, HEAP_PTR_OFFSET, Memory as MmuRef, VirtualAddress, PAGE_SIZE};
use vm::registers::Register;
use vm::vm::VM;

const MIN_KERNEL_MAP_BYTES: usize = 16 * 1024;
const KERNEL_STACK_BYTES: usize = 4 * PAGE_SIZE;

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
    memory: Rc<Memory>,
}

impl Bootloader {
    pub fn new(total_size_bytes: usize) -> Self {
        Self {
            config: BootConfig::default(),
            memory: Rc::new(Memory::new(total_size_bytes, PAGE_SIZE)),
        }
    }

    /// Load an ELF kernel image into a fresh page and return its entry point + backing memory.
    pub fn load_kernel(&mut self, elf_bytes: &[u8]) -> (u32, MmuRef) {
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

        assert!(
            image_end <= self.memory.size(),
            "ELF image does not fit in mapped memory (need {}, have {})",
            image_end,
            self.memory.size()
        );

        // Flatten code + rodata into a single buffer and write once to set heap pointer properly.
        let mut image = vec![0u8; image_size];
        let code_off = (code_base as usize).saturating_sub(min_base);
        image[code_off..code_off + code.len()].copy_from_slice(&code);
        if !rodata.is_empty() {
            let ro_off = (ro_base as usize).saturating_sub(min_base);
            image[ro_off..ro_off + rodata.len()].copy_from_slice(&rodata);
        }

        self.memory
            .map_range(VirtualAddress(min_base as u32), core::cmp::max(image_size, MIN_KERNEL_MAP_BYTES), Perms::rwx_kernel());
        self.memory
            .write_bytes(VirtualAddress(min_base as u32), &image);
        // Start the heap after the loaded image to avoid overwriting kernel text/rodata
        let heap_start = ((image_end + HEAP_PTR_OFFSET as usize + 7) & !7) as u32;
        self.memory.set_next_heap(VirtualAddress(heap_start));
        // Ensure the kernel has a mapped stack region near the top of memory.
        let stack_base = self
            .memory
            .stack_top()
            .as_usize()
            .saturating_sub(KERNEL_STACK_BYTES);
        self.memory
            .map_range(VirtualAddress(stack_base as u32), KERNEL_STACK_BYTES, Perms::rw_kernel());
        (entry_point, self.memory.clone() as MmuRef)
    }

    /// Execute a transaction bundle by delegating to the kernel. This mirrors the
    /// AVM entry point where the kernel is responsible for invoking programs.
    pub fn execute_bundle(
        &mut self,
        kernel_elf: &[u8],
        bundle: &TransactionBundle,
        state: Rc<RefCell<State>>,
        verbose: bool,
        verbose_writer: Option<Rc<RefCell<dyn FmtWrite>>>,
    ) {
        let (entry_point, memory) = self.load_kernel(kernel_elf);
        let host: Box<dyn vm::host_interface::HostInterface> = Box::new(NoopHost);

        let mut vm = VM::new(memory.clone(), host, Box::new(DefaultSyscallHandler::new(state.clone())));
        vm.cpu.verbose = verbose;
        if let Some(writer) = verbose_writer {
            vm.cpu.set_verbose_writer(writer);
        }
        vm.cpu.pc = entry_point;

        self.place_bundle(&mut vm, bundle);
        let encoded_state = state.borrow().encode();
        self.place_state(&mut vm, &encoded_state);
        self.place_boot_info(&mut vm);
        vm.raw_run();
    }

    fn place_bundle(&mut self, vm: &mut VM, bundle: &TransactionBundle) {
        let encoded = bundle.encode();
        let addr = vm.set_reg_to_data(Register::A0, &encoded);
        // Register a length hint so the kernel can bounds-check the payload.
        vm.set_reg_u32(Register::A1, encoded.len() as u32);
        // Keep heap aligned after our write.
        vm.memory
            .set_next_heap(VirtualAddress(
                (addr as usize + encoded.len() + HEAP_PTR_OFFSET as usize) as u32,
            ));
    }

    fn place_state(&mut self, vm: &mut VM, state: &[u8]) {
        let addr = vm.set_reg_to_data(Register::A2, state);
        vm.set_reg_u32(Register::A3, state.len() as u32);
        vm.memory
            .set_next_heap(VirtualAddress(
                (addr as usize + state.len() + HEAP_PTR_OFFSET as usize) as u32,
            ));
    }

    fn place_boot_info(&mut self, vm: &mut VM) {
        // For now the bootloader owns the page tables, so `root_ppn` is a placeholder (0).
        let boot_info = BootInfo::new(
            self.memory.current_root() as u32,
            vm.memory.stack_top().as_u32(),
            self.memory.size() as u32,
        );
        let bytes = unsafe {
            slice::from_raw_parts(
                &boot_info as *const BootInfo as *const u8,
                mem::size_of::<BootInfo>(),
            )
        };
        let addr = vm.set_reg_to_data(Register::A4, bytes);
        vm.memory
            .set_next_heap(VirtualAddress(
                (addr as usize + bytes.len() + HEAP_PTR_OFFSET as usize) as u32,
            ));
    }
}
