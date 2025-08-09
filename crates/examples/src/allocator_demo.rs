#![no_std]
#![no_main]

extern crate alloc;

use program::{entrypoint, types::result::Result, logf, types::address::Address, require, vm_panic};

/// Guest program that demonstrates heap allocation using VM syscalls
entrypoint!(main);
fn main(_self_address: Address, _caller: Address, _data: &[u8]) -> Result {
    // Need to import alloc types after entrypoint macro includes the allocator
    use alloc::vec::Vec;
    use alloc::string::String;
    use alloc::collections::BTreeMap;
        
    // Test Vec operations
    let mut numbers = Vec::new();
    numbers.push(12);
    numbers.push(15);
    numbers.push(100);

    match numbers.get(0) {
        Some(&n) => require(n == 12, b"First element should be 12"),
        None => vm_panic(b"Vec should not be empty"),
    }
    match numbers.get(1) {
        Some(&n) => require(n == 15, b"Second element should be 15"),
        None => vm_panic(b"Vec should not be empty"),
    }
    match numbers.get(2) {
        Some(&n) => require(n == 100, b"Third element should be 100"),
        None => vm_panic(b"Vec should not be empty"),
    }
    require(numbers.len() == 3, b"Vec should have 3 elements");

    // Test Vec with capacity
    let mut large_vec = Vec::with_capacity(50);
    for i in 0..20 {
        large_vec.push(i);
    }
    require(large_vec.len() == 20, b"Large vector should have 20 elements");
    
    // Test BTreeMap operations
    let mut scores = BTreeMap::new();
    scores.insert("alice", 95u32);
    scores.insert("bob", 87u32);
    scores.insert("charlie", 92u32);
    
    match scores.get("alice") {
        Some(&v) => require(v == 95u32, b"Alice's score should be 95"),
        None => vm_panic(b"Alice not found in BTreeMap"),
    }
    
    match scores.get("bob") {
        Some(&v) => require(v == 87u32, b"Bob's score should be 87"), 
        None => vm_panic(b"Bob not found in BTreeMap"),
    }
    
    require(scores.len() == 3, b"BTreeMap should have 3 elements");

    Result::new(true, 0)
}