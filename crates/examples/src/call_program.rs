#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Result, logf, require};
use program::call_contract::call;
use program::types::address::Address;    

fn my_vm_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    call(&_caller, &_caller, data);
    Result { success: true, error_code: 0 }
}

// EDUCATIONAL: Register the function as the entrypoint
// This macro tells the VM which function to call when the contract is invoked
entrypoint!(my_vm_entry);
