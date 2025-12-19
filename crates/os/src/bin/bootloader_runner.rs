use std::env;
use std::fs;

use bootloader::bootloader::Bootloader;
use types::address::Address;
use types::transaction::{Transaction, TransactionBundle, TransactionType};

fn main() {
    let kernel_path = env::args()
        .nth(1)
        .expect("pass a path to the kernel ELF as the first argument");
    let kernel_bytes = fs::read(&kernel_path).expect("failed to read kernel ELF");

    // Temporary: build a single transfer transaction for the bundle.
    let bundle = TransactionBundle::new(vec![Transaction {
        tx_type: TransactionType::Transfer,
        to: Address([0u8; 20]),
        from: Address([1u8; 20]),
        data: Vec::new(),
        value: 0,
        nonce: 0,
    }]);

    let mut bootloader = Bootloader::new(4, 4096);
    bootloader.execute_bundle(&kernel_bytes, &bundle);
}
