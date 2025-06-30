use std::collections::HashMap;
use crate::{StorageKey, StorageValue, Code};

#[derive(Clone, Debug)]
pub struct Account {
    pub nonce: u64,
    pub balance: u128,
    pub code: Option<Code>,
    pub storage: HashMap<StorageKey, StorageValue>,
}

impl Account {
    pub fn read(&self, key: &StorageKey) -> StorageValue {
        *self.storage.get(key).unwrap_or(&[0u8; 32])
    }

    pub fn write(&mut self, key: StorageKey, value: StorageValue) {
        self.storage.insert(key, value);
    }
}