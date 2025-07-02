// crates/storage/src/lib.rs

extern crate alloc;

use core::cell::RefCell;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Default)]
pub struct Storage {
    pub map: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Storage {
    /// Create new Storage with an empty map
    pub fn new() -> Self {
        Self::with_map(BTreeMap::new())
    }

    /// Create new Storage with a pre-populated map
    pub fn with_map(initial: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            map: RefCell::new(initial),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.map.borrow().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: Vec<u8>) {
        self.map.borrow_mut().insert(key.to_string(), value);
    }
}
