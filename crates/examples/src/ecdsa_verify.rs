#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

use program::{entrypoint, require, ecdsa, types};
use types::address::Address;
use types::result::Result;

entrypoint!(ecdsa_verify_entry);

fn ecdsa_verify_entry(_self_address: Address, _caller: Address, data: &[u8]) -> Result {
    // Input format: pubkey (33 or 65 bytes) + signature (64 bytes) + message_hash (32 bytes)

    // Validate minimum input length (33 + 64 + 32 = 129 bytes for compressed pubkey)
    require(data.len() >= 129, b"Insufficient input data");

    let pubkey_len = if data[0] == 0x04 { 65 } else { 33 };
    require(data.len() >= pubkey_len + 64 + 32, b"Invalid input length");

    let pubkey = &data[0..pubkey_len];
    let signature = &data[pubkey_len..pubkey_len + 64];
    let message_hash = &data[pubkey_len + 64..pubkey_len + 64 + 32];

    // Verify the signature
    match ecdsa::verify_signature(pubkey, signature, message_hash) {
        Ok(()) => {
            // Signature is valid - return 1
            Result::with_u32(1)
        }
        Err(_) => {
            // Signature is invalid - return 0
            Result::with_u32(0)
        }
    }
}