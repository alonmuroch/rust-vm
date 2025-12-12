use types::address::Address;

#[derive(Debug, Clone)]
pub enum TransactionType {
    /// Type 0 - Regular value transfer (not a contract)
    Transfer = 0,

    /// Type 1 - Account create with program data (contract deployment)
    CreateAccount = 1,

    /// Type 2 - Contract call (calling into existing code)
    ProgramCall = 2,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_type: TransactionType, // type of transaction
    pub to: Address,              // recipient address
    pub from: Address,            // sender public key/address
    pub data: Vec<u8>,            // input data
    pub value: u64,               // amount/value sent
    pub nonce: u64,               // transaction nonce
}

/// Holds a set of transactions to be processed as a unit
#[derive(Debug, Clone)]
pub struct TransactionBundle {
    pub transactions: Vec<Transaction>,
}

impl TransactionBundle {
    pub fn new(transactions: Vec<Transaction>) -> Self {
        TransactionBundle { transactions }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}
