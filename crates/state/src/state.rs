use alloc::vec::Vec;
use crate::account::Account;
// This file defines the state of the program, which includes a collection of accounts.
// It provides a structure to hold accounts and their balances, and allows for easy management of these accounts.
// It is used to maintain the state of the program across different transactions and operations.
// It is essential for tracking the balances and other properties of accounts in the system.                                                        

#[derive(Default)]
pub struct State {
    pub accounts: Vec<Account>,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }

    pub fn add_account(&mut self, balance: u64) -> usize {
        let index = self.accounts.len();
        self.accounts.push(Account { index, balance });
        index
    }

    pub fn get_account(&self, index: usize) -> Option<&Account> {
        self.accounts.get(index)
    }

    pub fn get_account_mut(&mut self, index: usize) -> Option<&mut Account> {
        self.accounts.get_mut(index)
    }
}