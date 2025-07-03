// crates/storage/src/lib.rs

extern crate alloc;

use core::cell::RefCell;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;

/// Represents persistent storage for the blockchain virtual machine.
/// 
/// EDUCATIONAL PURPOSE: This struct provides a simple key-value storage
/// system that simulates how blockchain systems store persistent data.
/// In real blockchains, this would be implemented using databases, file
/// systems, or distributed storage systems.
/// 
/// STORAGE CONCEPTS:
/// - Key-value pairs for simple data storage
/// - Persistent across VM restarts
/// - Thread-safe access using RefCell
/// - Ordered storage using BTreeMap
/// 
/// REAL-WORLD BLOCKCHAIN COMPARISON:
/// In Ethereum, storage is organized differently:
/// - Each contract has its own storage space
/// - Storage is accessed via 32-byte keys and values
/// - Storage is part of the global state trie
/// - Gas costs are associated with storage operations
/// 
/// MEMORY MANAGEMENT: Uses BTreeMap for ordered iteration and efficient
/// lookups. The RefCell wrapper provides interior mutability, allowing
/// the storage to be modified even when borrowed immutably.
/// 
/// PERSISTENCE: This is an in-memory implementation. In production,
/// data would be persisted to disk or a database to survive system
/// restarts and crashes.
/// 
/// THREAD SAFETY: The current implementation is not thread-safe. In a
/// real blockchain, storage would need to handle concurrent access
/// from multiple transactions and validators.
#[derive(Debug, Default)]
pub struct Storage {
    /// Internal key-value storage using BTreeMap for ordered iteration
    /// 
    /// EDUCATIONAL: BTreeMap provides O(log n) lookups and ordered
    /// iteration, which is useful for debugging and deterministic
    /// behavior. RefCell allows interior mutability while maintaining
    /// Rust's borrowing rules at runtime.
    pub map: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Storage {
    /// Creates a new empty storage instance.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates storage initialization.
    /// In blockchain systems, storage is typically initialized empty
    /// and populated as transactions are processed.
    /// 
    /// MEMORY ALLOCATION: Creates an empty BTreeMap that will grow
    /// as data is added. This is memory-efficient for systems that
    /// start with little data.
    /// 
    /// USAGE: Use this when starting a new blockchain or when
    /// resetting storage for testing purposes.
    pub fn new() -> Self {
        Self::with_map(BTreeMap::new())
    }

    /// Creates a new storage instance with pre-populated data.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how storage can be
    /// initialized with existing data, such as when loading from
    /// a database or file system.
    /// 
    /// USAGE: Useful for testing with known data sets or when
    /// restoring storage from a backup or snapshot.
    /// 
    /// PARAMETERS:
    /// - initial: Pre-existing key-value pairs to populate storage
    pub fn with_map(initial: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            map: RefCell::new(initial),
        }
    }

    /// Retrieves a value from storage by key.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates safe storage access.
    /// Returns None if the key doesn't exist, which is common in
    /// blockchain systems where storage is sparse.
    /// 
    /// MEMORY SAFETY: Uses RefCell::borrow() to get immutable access
    /// to the storage map. This ensures thread safety at runtime.
    /// 
    /// CLONING: The value is cloned to avoid lifetime issues. In a
    /// production system, you might use references or more sophisticated
    /// memory management to avoid copying large values.
    /// 
    /// PARAMETERS:
    /// - key: The storage key to look up
    /// 
    /// RETURNS: Some(value) if the key exists, None otherwise
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.map.borrow().get(key).cloned()
    }

    /// Stores a value in storage with the specified key.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates persistent storage updates.
    /// In blockchain systems, storage changes are part of the transaction
    /// and are committed atomically with the rest of the state changes.
    /// 
    /// MUTABILITY: Uses RefCell::borrow_mut() to get mutable access
    /// to the storage map. This allows modification while maintaining
    /// Rust's borrowing rules.
    /// 
    /// KEY OWNERSHIP: The key is converted to a String to ensure
    /// ownership. In a production system, you might use more efficient
    /// key representations or avoid unnecessary allocations.
    /// 
    /// PARAMETERS:
    /// - key: The storage key
    /// - value: The data to store
    pub fn set(&self, key: &str, value: Vec<u8>) {
        self.map.borrow_mut().insert(key.to_string(), value);
    }
}
