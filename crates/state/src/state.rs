use std::rc::Rc;
use std::collections::HashMap;
use storage::Storage;
use crate::{Account};
use types::address::Address;

#[derive(Clone, Debug)]
pub struct State {
    pub accounts: HashMap<Address, Account>,
}

impl State {
    pub fn new() -> Self {
        Self { accounts: HashMap::new() }
    }

    /// Construct a State from an existing Storage instance
    pub fn new_from_storage(_storage: Rc<Storage>) -> Self {
        Self { accounts: HashMap::new() }
    }

    pub fn get_account(&self, addr: &Address) -> Option<&Account> {
        self.accounts.get(addr)
    }

    pub fn get_account_mut(&mut self, addr: &Address) -> &mut Account {
        self.accounts.entry(*addr).or_insert_with(|| Account {
            nonce: 0,
            balance: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        })
    }

    pub fn is_contract(&self, _addr: Address) -> bool {
        // self.accounts.get(addr).map_or(false, |acc| acc.code.is_some())
        return true;
    }   

    pub fn deploy_contract(&mut self, addr: Address, code: Vec<u8>) {
        let acc = self.accounts.entry(addr).or_insert_with(|| Account {
            nonce: 0,
            balance: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        });
        acc.code = code;
    }
}
