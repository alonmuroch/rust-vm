use std::rc::Rc;
use std::collections::HashMap;
use storage::Storage;
use crate::{Address, Account, Code};

#[derive(Clone, Debug)]
pub struct State {
    pub accounts: HashMap<Address, Account>,
}

impl State {
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
            code: None,
            storage: HashMap::new(),
        })
    }

    pub fn deploy_contract(&mut self, addr: Address, code: Code) {
        let acc = self.accounts.entry(addr).or_insert_with(|| Account {
            nonce: 0,
            balance: 0,
            code: None,
            storage: HashMap::new(),
        });
        acc.code = Some(code);
    }
}
