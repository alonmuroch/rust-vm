use std::{cell::RefCell, rc::Rc};

use vm::instruction::Instruction;
use vm::metering::{MemoryAccessKind, MeterResult, Metering};
use vm::sys_call::{
    SYSCALL_ALLOC, SYSCALL_BALANCE, SYSCALL_CALL_PROGRAM, SYSCALL_DEALLOC, SYSCALL_FIRE_EVENT,
    SYSCALL_LOG, SYSCALL_STORAGE_GET, SYSCALL_STORAGE_SET, SYSCALL_TRANSFER,
};

/// Gas pricing table inspired by the EVM schedule:
/// - Storage operations are expensive (SLOAD/SSTORE style)
/// - External calls and value transfers carry a base "call" charge
/// - Memory copies and logs are charged per byte like calldata/logdata gas
#[derive(Debug, Clone, Copy)]
pub struct GasSchedule {
    pub instruction: u64,
    pub memory_load_base: u64,
    pub memory_store_base: u64,
    pub memory_atomic_base: u64,
    pub memory_res_load_base: u64,
    pub memory_res_store_base: u64,
    pub memory_byte_cost: u64,
    pub register_read: u64,
    pub register_write: u64,
    pub pc_update: u64,
    pub syscall_base: u64,
    pub syscall_storage_get: u64,
    pub syscall_storage_set: u64,
    pub syscall_log: u64,
    pub syscall_call_program: u64,
    pub syscall_fire_event: u64,
    pub syscall_alloc: u64,
    pub syscall_dealloc: u64,
    pub syscall_transfer: u64,
    pub syscall_balance: u64,
    pub call_base: u64,
    pub call_data_byte: u64,
    pub log_data_byte: u64,
    pub storage_key_byte: u64,
    pub storage_value_byte: u64,
    pub alloc_word: u64,
    pub alloc_base: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        // Costs take cues from Ethereum (London):
        // - CALL base ~700, value transfer ~9000, cold storage access ~2100, SSTORE ~20k
        // - Calldata/log data charged per byte; memory growth charged per word
        Self {
            instruction: 1,
            memory_load_base: 3,
            memory_store_base: 5,
            memory_atomic_base: 25,
            memory_res_load_base: 12,
            memory_res_store_base: 18,
            memory_byte_cost: 1,
            register_read: 0,
            register_write: 0,
            pc_update: 0,
            syscall_base: 30,
            syscall_storage_get: 2100,
            syscall_storage_set: 20_000,
            syscall_log: 375,
            syscall_call_program: 40, // bulk of the cost is charged via call_base/call_data_byte
            syscall_fire_event: 375,
            syscall_alloc: 15,
            syscall_dealloc: 4,
            syscall_transfer: 9000,
            syscall_balance: 2600,
            call_base: 700,
            call_data_byte: 4,
            log_data_byte: 8,
            storage_key_byte: 4,
            storage_value_byte: 16,
            alloc_word: 3,
            alloc_base: 15,
        }
    }
}

impl GasSchedule {
    fn memory_cost(&self, kind: MemoryAccessKind, bytes: usize) -> u64 {
        let per_byte = self.memory_byte_cost.saturating_mul(bytes as u64);
        let base = match kind {
            MemoryAccessKind::Load => self.memory_load_base,
            MemoryAccessKind::Store => self.memory_store_base,
            MemoryAccessKind::Atomic => self.memory_atomic_base,
            MemoryAccessKind::ReservationLoad => self.memory_res_load_base,
            MemoryAccessKind::ReservationStore => self.memory_res_store_base,
        };
        base.saturating_add(per_byte)
    }

    fn syscall_cost(&self, call_id: u32) -> u64 {
        let specific = match call_id {
            SYSCALL_STORAGE_GET => self.syscall_storage_get,
            SYSCALL_STORAGE_SET => self.syscall_storage_set,
            SYSCALL_LOG => self.syscall_log,
            SYSCALL_CALL_PROGRAM => self.syscall_call_program,
            SYSCALL_FIRE_EVENT => self.syscall_fire_event,
            SYSCALL_ALLOC => self.syscall_alloc,
            SYSCALL_DEALLOC => self.syscall_dealloc,
            SYSCALL_TRANSFER => self.syscall_transfer,
            SYSCALL_BALANCE => self.syscall_balance,
            _ => 0,
        };
        self.syscall_base.saturating_add(specific)
    }

    fn syscall_data_cost(&self, call_id: u32, bytes: usize) -> u64 {
        let per_byte = match call_id {
            SYSCALL_STORAGE_GET => self.storage_key_byte,
            SYSCALL_STORAGE_SET => self.storage_value_byte,
            SYSCALL_LOG | SYSCALL_FIRE_EVENT => self.log_data_byte,
            SYSCALL_TRANSFER | SYSCALL_BALANCE => self.storage_key_byte,
            _ => self.call_data_byte,
        };
        per_byte.saturating_mul(bytes as u64)
    }

    fn alloc_cost(&self, bytes: usize) -> u64 {
        let words = ((bytes as u64).saturating_add(31)) / 32;
        self.alloc_base
            .saturating_add(self.alloc_word.saturating_mul(words.max(1)))
    }

    fn call_cost(&self, input_bytes: usize) -> u64 {
        self.call_base
            .saturating_add(self.call_data_byte.saturating_mul(input_bytes as u64))
    }
}

/// Core gas accounting state shared across nested contract calls.
#[derive(Debug)]
pub struct GasMeter {
    schedule: GasSchedule,
    gas_used: u64,
}

impl GasMeter {
    pub fn new() -> Self {
        Self {
            schedule: GasSchedule::default(),
            gas_used: 0,
        }
    }

    pub fn used(&self) -> u64 {
        self.gas_used
    }

    pub fn charge_call(&mut self, input_bytes: usize) -> MeterResult {
        self.consume(self.schedule.call_cost(input_bytes))
    }

    fn consume(&mut self, amount: u64) -> MeterResult {
        if amount == 0 {
            return MeterResult::Continue;
        }

        self.gas_used = self.gas_used.saturating_add(amount);
        MeterResult::Continue
    }

    fn charge_instruction(&mut self) -> MeterResult {
        self.consume(self.schedule.instruction)
    }

    fn charge_memory(&mut self, kind: MemoryAccessKind, bytes: usize) -> MeterResult {
        self.consume(self.schedule.memory_cost(kind, bytes))
    }

    fn charge_syscall(&mut self, call_id: u32) -> MeterResult {
        self.consume(self.schedule.syscall_cost(call_id))
    }

    fn charge_syscall_data(&mut self, call_id: u32, bytes: usize) -> MeterResult {
        self.consume(self.schedule.syscall_data_cost(call_id, bytes))
    }

    fn charge_register_read(&mut self) -> MeterResult {
        self.consume(self.schedule.register_read)
    }

    fn charge_register_write(&mut self) -> MeterResult {
        self.consume(self.schedule.register_write)
    }

    fn charge_pc_update(&mut self) -> MeterResult {
        self.consume(self.schedule.pc_update)
    }

    fn charge_alloc(&mut self, bytes: usize) -> MeterResult {
        self.consume(self.schedule.alloc_cost(bytes))
    }
}

/// Thin adapter that lets the VM/CPU hold a boxed metering implementation
/// while multiple VMs share the same underlying gas counter.
#[derive(Clone)]
pub struct SharedGasMeter {
    inner: Rc<RefCell<GasMeter>>,
}

impl SharedGasMeter {
    pub fn new(inner: Rc<RefCell<GasMeter>>) -> Self {
        Self { inner }
    }

    pub fn used(&self) -> u64 {
        self.inner.borrow().used()
    }
}

impl std::fmt::Debug for SharedGasMeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let borrowed = self.inner.borrow();
        f.debug_struct("SharedGasMeter")
            .field("used", &borrowed.used())
            .finish()
    }
}

impl Metering for SharedGasMeter {
    fn on_instruction(&mut self, _pc: u32, _instr: &Instruction, _size: u8) -> MeterResult {
        self.inner.borrow_mut().charge_instruction()
    }

    fn on_memory_access(
        &mut self,
        kind: MemoryAccessKind,
        _addr: usize,
        bytes: usize,
    ) -> MeterResult {
        self.inner.borrow_mut().charge_memory(kind, bytes)
    }

    fn on_syscall(&mut self, call_id: u32, _args: &[u32; 6]) -> MeterResult {
        self.inner.borrow_mut().charge_syscall(call_id)
    }

    fn on_syscall_data(&mut self, call_id: u32, bytes: usize) -> MeterResult {
        self.inner.borrow_mut().charge_syscall_data(call_id, bytes)
    }

    fn on_register_read(&mut self, _reg: usize) -> MeterResult {
        self.inner.borrow_mut().charge_register_read()
    }

    fn on_register_write(&mut self, _reg: usize) -> MeterResult {
        self.inner.borrow_mut().charge_register_write()
    }

    fn on_pc_update(&mut self, _old_pc: u32, _new_pc: u32) -> MeterResult {
        self.inner.borrow_mut().charge_pc_update()
    }

    fn on_alloc(&mut self, bytes: usize) -> MeterResult {
        self.inner.borrow_mut().charge_alloc(bytes)
    }

    fn on_call(&mut self, input_bytes: usize) -> MeterResult {
        self.inner.borrow_mut().charge_call(input_bytes)
    }
}
