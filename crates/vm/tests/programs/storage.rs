#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Pubkey, Result};
use program::persist_struct;
use core::convert::TryInto;


// Macro must be invoked before you use `Counter`
persist_struct!(Counter, b"counter", {
    value: u64,
});

fn my_vm_entry(_caller: Pubkey, _data: &[u8]) -> Result {
    // Now Counter is in scope
    let mut counter = Counter::load().unwrap_or(Counter { value: 0 });

    counter.value += 1;
    counter.store();

    Result { success: true, error_code: counter.value as u32 }
}

entrypoint!(my_vm_entry);