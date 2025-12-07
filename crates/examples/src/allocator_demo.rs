#![no_std]
#![no_main]

extern crate alloc;

use program::{entrypoint, types::result::Result, types::address::Address, require, vm_panic};
use core::convert::TryInto;

/// Guest program that demonstrates heap allocation using VM syscalls
entrypoint!(main);
fn main(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Need to import alloc types after entrypoint macro includes the allocator
    use alloc::vec::Vec;
    use alloc::collections::BTreeMap;

    // Expect at least 6 u32 values in little-endian form:
    // - first 3 populate the Vec
    // - next 3 populate the BTreeMap values for alice/bob/charlie
    const WORDS: usize = 6;
    if data.len() < WORDS * 4 {
        vm_panic(b"insufficient data for allocator demo");
    }

    let read_word = |i: usize| -> u32 {
        let start = i * 4;
        let bytes: [u8; 4] = data[start..start + 4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    };

    let n0 = read_word(0);
    let n1 = read_word(1);
    let n2 = read_word(2);

    let alice_score = read_word(3);
    let bob_score = read_word(4);
    let charlie_score = read_word(5);

    // Test Vec operations
    let mut numbers = Vec::new();
    numbers.push(n0);
    numbers.push(n1);
    numbers.push(n2);

    match numbers.get(0) {
        Some(&n) => require(n == n0, b"First element mismatch"),
        None => vm_panic(b"Vec should not be empty"),
    }
    match numbers.get(1) {
        Some(&n) => require(n == n1, b"Second element mismatch"),
        None => vm_panic(b"Vec should not be empty"),
    }
    match numbers.get(2) {
        Some(&n) => require(n == n2, b"Third element mismatch"),
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
    scores.insert("alice", alice_score);
    scores.insert("bob", bob_score);
    scores.insert("charlie", charlie_score);
    
    match scores.get("alice") {
        Some(&v) => require(v == alice_score, b"Alice's score mismatch"),
        None => vm_panic(b"Alice not found in BTreeMap"),
    }
    
    match scores.get("bob") {
        Some(&v) => require(v == bob_score, b"Bob's score mismatch"), 
        None => vm_panic(b"Bob not found in BTreeMap"),
    }

    match scores.get("charlie") {
        Some(&v) => require(v == charlie_score, b"Charlie's score mismatch"),
        None => vm_panic(b"Charlie not found in BTreeMap"),
    }
    
    require(scores.len() == 3, b"BTreeMap should have 3 elements");

    Result::new(true, 0)
}
