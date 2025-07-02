use std::rc::Rc;
use std::collections::HashMap;
use storage::Storage;
use crate::{Account};
use types::address::Address;
use hex::encode as hex_encode;
use alloc::collections::BTreeMap;

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
            is_contract: false,
            storage: BTreeMap::new(),
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
            is_contract: false,
            storage: BTreeMap::new(),
        });
        acc.code = code;
        acc.is_contract = true;
    }

    pub fn pretty_print(&self) {
        println!("--- State Dump ---");
        for (addr, acc) in &self.accounts {
            println!("  🔑 Address: 0x{}", hex_encode(addr.0));
            println!("      - Balance: {}", acc.balance);
            println!("      - Nonce: {}", acc.nonce);
            println!("      - Is contract?: {}", acc.is_contract);
            println!("      - Code size: {} bytes", acc.code.len());
            println!("      - Storage:");
            for (key, value) in &acc.storage {
                let value_hex: Vec<String> = value.iter().map(|b| format!("{:02x}", b)).collect();
                println!("          Key: {:<20} | Value ({} bytes): {}", key, value.len(), value_hex.join(" "));
            }
            println!();
        }
        println!("--------------------");
    }
}
