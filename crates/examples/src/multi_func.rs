#![no_std]
#![no_main]

extern crate program;

use program::{entrypoint, Result, vm_panic, require};
use program::types::address::Address;
use program::router::{route, FuncCall};

fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    route(data, 4, |call| match call.selector {
        0x01 => compare(call.args),
        0x02 => other(call.args),
        _ => vm_panic(b"unknown selector"),
    })
}

fn compare(data: &[u8]) -> Result {
    require(data.len() == 8, b"compare expects 8 bytes");

    let mut a = [0u8; 4];
    let mut b = [0u8; 4];
    a.copy_from_slice(&data[0..4]);
    b.copy_from_slice(&data[4..8]);

    let a = u32::from_le_bytes(a);
    let b = u32::from_le_bytes(b);

    if a > b {
        Result { success: true, error_code: a }
    } else {
        Result { success: false, error_code: b }
    }
}

fn other(_data: &[u8]) -> Result {
    vm_panic(b"Intentional failure");
}

entrypoint!(my_vm_entry);
