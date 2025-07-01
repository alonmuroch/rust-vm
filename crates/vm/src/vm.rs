use std::rc::Rc;
use std::task::Context;
use crate::cpu::CPU;
use crate::registers::Register;
use crate::memory_page::{MemoryPage};
use crate::global::Config;
use crate::transaction::Transaction;
use crate::context::{ExecutionContext};
use storage::{Storage};
use state::State;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub struct VM {
    pub cpu: CPU,
    pub memory: MemoryPage,
    pub storage: Rc<Storage>,
    pub state: State,
    pub context: ExecutionContext,
}

impl VM {
    pub fn new(memory_size: usize, context: ExecutionContext) -> Self {
        let memory = MemoryPage::new(memory_size);

        let mut regs = [0u32; 32];
        regs[Register::Sp as usize] = memory.stack_top();

        let cpu = CPU {
            pc: 0,
            regs,
            verbose: false,
        };

        let storage = Rc::new(Storage::new());
        let state = State::new_from_storage(Rc::clone(&storage));

        Self { cpu, memory, storage, state, context: context}
    }

    pub fn set_code(&mut self, addr: usize, code: &[u8]) {
        self.memory.write_code(addr, code);
        self.cpu.pc = addr as u32;
    }


    pub fn set_rodata(&mut self, addr: usize, data: &[u8]) {
        self.memory.write_rodata(addr, data);
    }

    pub fn alloc_and_write(&mut self, data: &[u8]) -> u32 {
        self.memory.alloc_on_heap(data)
    }

    pub fn set_reg_to_data(&mut self, reg: Register, data: &[u8]) -> u32 {
        let addr = self.alloc_and_write(data);
        self.cpu.regs[reg as usize] = addr;

        println!(
            "📥 set reg x{} to addr 0x{:08x} (len = {})",
            reg as u32,
            addr,
            data.len()
        );
        println!(
            "📦 data written to 0x{:08x}: {:02x?}",
            addr,
            data
        );

        addr
    }

    pub fn set_reg_u32(&mut self, reg: Register, data: u32) {
        self.cpu.regs[reg as usize] = data;
    }

    pub fn dump_memory(&self, start: usize, end: usize) {
        assert!(start < end, "invalid memory range");
        assert!(end <= self.memory.mem().len(), "range out of bounds");

        let next_heap = self.memory.next_heap.get();
        println!("--- Memory Dump ---");
        println!("Next heap pointer: 0x{:08x}", next_heap);

        for addr in (start..end).step_by(16) {
            let line = &self.memory.mem()[addr..end.min(addr + 16)];

            let hex: Vec<String> = line.iter().map(|b| format!("{:02x}", b)).collect();
            let hex_str = hex.join(" ");

            let ascii: String = line.iter()
                .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                .collect();

            println!("{:08x}  {:<47}  |{}|", addr, hex_str, ascii);
        }
        println!("-------------------");
    }


    pub fn dump_storage(&self) {
        println!("--- Storage Dump ---");
        for (key, value) in self.storage.map.borrow().iter() {
            let key_str = key;
            let value_hex: Vec<String> = value.iter().map(|b| format!("{:02x}", b)).collect();
            println!("Key: {:<20} | Value ({} bytes): {}", key_str, value.len(), value_hex.join(" "));
        }
        println!("--------------------");
    }

    pub fn dump_registers(&self) {
        println!("--- Register Dump ---");

        const ABI_NAMES: [&str; 32] = [
            "zero", "ra",  "sp",  "gp",  "tp",  "t0",  "t1",  "t2",
            "s0",   "s1",  "a0",  "a1",  "a2",  "a3",  "a4",  "a5",
            "a6",   "a7",  "s2",  "s3",  "s4",  "s5",  "s6",  "s7",
            "s8",   "s9",  "s10", "s11", "t3",  "t4",  "t5",  "t6",
        ];

        for i in 0..32 {
            let name = ABI_NAMES[i];
            let val = self.cpu.regs[i];
            println!("x{:02} ({:<4}) = 0x{:08x} ({})", i, name, val, val);
        }

        println!("pc           = 0x{:08x}", self.cpu.pc);
        println!("------------------------");
    }

    pub fn run_tx(&mut self, tx: Transaction) -> u32 {
        // to address
        let _address_ptr: u32 = self.set_reg_to_data(Register::A0, &tx.to);

        // from address
        let _pubkey_ptr = self.set_reg_to_data(Register::A1, &tx.from);

        // input data
        let input_len = tx.data.len();
        if input_len > Config::MAX_INPUT_LEN {
            panic!(
                "Entrypoint: input length {} exceeds MAX_INPUT_LEN ({})",
                input_len,
                Config::MAX_INPUT_LEN
            );
        }
        let _input_ptr = self.set_reg_to_data(Register::A2, &tx.data);
        self.set_reg_u32(Register::A3, input_len as u32);

        // result pointer
        let result_ptr = self.set_reg_to_data(Register::A4, &[0u8; 5]);

        // run the VM
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.raw_run(); // this might panic
        }));
        if let Err(e) = result {
            eprintln!("💥 VM panicked: {:?}", e);
            panic!("VM panicked");
        }

        return result_ptr;
    }

    /// Starts program execution without initializing registers or setting up state.
    /// This assumes the VM is already configured and simply jumps to the program counter.
    fn raw_run(&mut self) {
        while self.cpu.step(&self.memory, &self.storage, &self.context) {}
    }
} 