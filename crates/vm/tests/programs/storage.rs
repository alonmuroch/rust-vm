#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Pubkey, Result};
use program::persist_struct;
use core::convert::TryInto;

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_KEY: [u8; 8] = *b"counter\0";

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_KEY2: [u8; 9] = *b"counter2\0";

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_KEY3: [u8; 10] = *b"counter33\0";

persist_struct!(Counter, PERSIST_KEY3, {
    value: u64,
});

fn my_vm_entry(_caller: Pubkey, _data: &[u8]) -> Result {
    // Now Counter is in scope
    let mut counter = Counter::load().unwrap_or(Counter { value: 0 });

    counter.value += 5;
    counter.store();

    Result { success: true, error_code: counter.value as u32 }
}

entrypoint!(my_vm_entry);