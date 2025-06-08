use std::collections::HashMap;

pub struct State {
    pub storage: HashMap<String, u64>,
}

impl State {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        self.storage.get(key).copied()
    }

    pub fn set(&mut self, key: String, value: u64) {
        self.storage.insert(key, value);
    }
}
