use core::slice;
use std::convert::TryInto;

use avm::transaction::{Transaction, TransactionBundle, TransactionType};
use types::address::Address;

/// Kernel entrypoint. Receives a pointer/length pair to an encoded `TransactionBundle`
/// (produced by the bootloader) and walks each transaction.
#[unsafe(no_mangle)]
pub extern "C" fn _start(bundle_ptr: *const u8, bundle_len: usize) -> ! {
    let encoded = unsafe { slice::from_raw_parts(bundle_ptr, bundle_len) };

    if let Some(bundle) = decode_bundle(encoded) {
        for tx in &bundle.transactions {
            execute_transaction(tx);
        }
    }

    // In a real kernel this would never return; loop forever for now.
    loop {}
}

fn execute_transaction(_tx: &Transaction) {
    // TODO: dispatch to programs; for now this is a stub.
}

fn decode_bundle(encoded: &[u8]) -> Option<TransactionBundle> {
    let mut cursor = 0usize;

    let mut read = |len: usize| -> Option<&[u8]> {
        if cursor + len > encoded.len() {
            return None;
        }
        let slice = &encoded[cursor..cursor + len];
        cursor += len;
        Some(slice)
    };

    let tx_count_bytes = read(4)?;
    let tx_count = u32::from_le_bytes(tx_count_bytes.try_into().ok()?) as usize;
    let mut transactions = Vec::with_capacity(tx_count);

    for _ in 0..tx_count {
        let tx_type_byte = *read(1)?.first()?;
        let tx_type = match tx_type_byte {
            0 => TransactionType::Transfer,
            1 => TransactionType::CreateAccount,
            2 => TransactionType::ProgramCall,
            _ => return None,
        };

        let mut to = [0u8; 20];
        to.copy_from_slice(read(20)?);
        let mut from = [0u8; 20];
        from.copy_from_slice(read(20)?);

        let data_len_bytes = read(4)?;
        let data_len = u32::from_le_bytes(data_len_bytes.try_into().ok()?) as usize;
        let data = read(data_len)?.to_vec();

        let value_bytes = read(8)?;
        let value = u64::from_le_bytes(value_bytes.try_into().ok()?);

        let nonce_bytes = read(8)?;
        let nonce = u64::from_le_bytes(nonce_bytes.try_into().ok()?);

        transactions.push(Transaction {
            tx_type,
            to: Address(to),
            from: Address(from),
            data,
            value,
            nonce,
        });
    }

    Some(TransactionBundle { transactions })
}
