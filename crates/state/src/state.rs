use std::rc::Rc;
use std::collections::HashMap;
use storage::Storage;
use crate::{Account};
use types::address::Address;
use hex::encode as hex_encode;
use alloc::collections::BTreeMap;

/// Represents the global state of the blockchain virtual machine.
/// 
/// EDUCATIONAL PURPOSE: This struct manages all accounts in the blockchain,
/// similar to how Ethereum's state trie works. It's the central data structure
/// that tracks all accounts, their balances, code, and storage.
/// 
/// BLOCKCHAIN STATE CONCEPTS:
/// - Each address can have an account
/// - Accounts can be regular (hold value) or contracts (hold code)
/// - All state changes are atomic (all succeed or all fail)
/// - State persists between transactions
/// 
/// REAL-WORLD BLOCKCHAIN COMPARISON:
/// This is a simplified version of Ethereum's state management. In Ethereum:
/// - State is stored in a Merkle Patricia Trie for efficient proofs
/// - Accounts have additional fields like storage root and code hash
/// - State changes are tracked for rollback capability
/// - Gas costs are associated with state operations
/// 
/// DATA STRUCTURE: Uses a HashMap for O(1) account lookups by address.
/// In production blockchains, this would be a more sophisticated data structure
/// like a Merkle tree for efficient state proofs.
/// 
/// MEMORY MANAGEMENT: All accounts are kept in memory for fast access.
/// In production systems, only frequently accessed accounts would be in memory,
/// with the rest stored on disk or in a database.
/// 
/// THREAD SAFETY: This implementation is not thread-safe. In a real blockchain,
/// the state would need to handle concurrent access from multiple transactions
/// and validators.
/// 
/// PERSISTENCE: The state can be reconstructed from storage, though the current
/// implementation is simplified. Real blockchains use sophisticated persistence
/// mechanisms to ensure data durability and recovery.
#[derive(Clone, Debug)]
pub struct State {
    /// Maps addresses to their corresponding accounts.
    /// 
    /// EDUCATIONAL: This is the core data structure that represents the
    /// entire blockchain state. Each entry contains an account with its
    /// balance, code, storage, and other metadata.
    pub accounts: HashMap<Address, Account>,
}

impl State {
    /// Creates a new empty state.
    /// 
    /// EDUCATIONAL PURPOSE: This represents the initial state of a blockchain
    /// before any transactions have been processed. In real blockchains,
    /// there might be genesis accounts with initial balances.
    /// 
    /// USAGE: Typically called when starting a new blockchain or when
    /// resetting the state for testing purposes.
    pub fn new() -> Self {
        Self { accounts: HashMap::new() }
    }

    /// Constructs a State from an existing Storage instance.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how state can be reconstructed
    /// from persistent storage. In real blockchains, the state is often
    /// stored on disk and loaded into memory when needed.
    /// 
    /// NOTE: This is currently a placeholder implementation that always
    /// returns an empty state. In a real system, this would deserialize
    /// the state from the provided storage.
    pub fn new_from_storage(_storage: Rc<Storage>) -> Self {
        Self { accounts: HashMap::new() }
    }

    /// Retrieves an account by address (immutable reference).
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates safe account access for reading.
    /// Returns None if the account doesn't exist, which is common in
    /// blockchain systems where addresses might not have accounts yet.
    /// 
    /// USAGE: Use this when you need to read account data but not modify it.
    /// This is the preferred method for read-only operations.
    /// 
    /// PARAMETERS:
    /// - addr: The address of the account to retrieve
    /// 
    /// RETURNS: Some(account) if the account exists, None otherwise
    pub fn get_account(&self, addr: &Address) -> Option<&Account> {
        self.accounts.get(addr)
    }

    /// Retrieves an account by address (mutable reference), creating it if it doesn't exist.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates account creation on-demand.
    /// In blockchain systems, accounts are often created implicitly when
    /// they first receive a transaction or are called.
    /// 
    /// ACCOUNT CREATION: If the account doesn't exist, it creates a new one
    /// with default values (0 balance, 0 nonce, no code, not a contract).
    /// 
    /// LAZY INITIALIZATION: This pattern is common in blockchain systems
    /// because it saves storage space - accounts only exist when they're
    /// actually used. This is different from traditional databases where
    /// you might pre-allocate space for all possible accounts.
    /// 
    /// DEFAULT VALUES EXPLANATION:
    /// - nonce: 0 - No transactions have been sent from this account yet
    /// - balance: 0 - No funds have been transferred to this account
    /// - code: Vec::new() - No smart contract code deployed
    /// - is_contract: false - This is a regular account, not a contract
    /// - storage: BTreeMap::new() - No persistent storage allocated
    /// 
    /// MEMORY EFFICIENCY: Using BTreeMap for storage provides ordered
    /// iteration and efficient lookups while using less memory than HashMap
    /// for small datasets.
    /// 
    /// USAGE: Use this when you need to modify account data (e.g., update
    /// balance, deploy code, modify storage).
    /// 
    /// PARAMETERS:
    /// - addr: The address of the account to retrieve or create
    /// 
    /// RETURNS: Mutable reference to the account (guaranteed to exist)
    pub fn get_account_mut(&mut self, addr: &Address) -> &mut Account {
        self.accounts.entry(*addr).or_insert_with(|| Account {
            nonce: 0,                    // No transactions yet
            balance: 0,                  // No initial balance
            code: Vec::new(),            // No code (not a contract)
            is_contract: false,          // Regular account
            storage: BTreeMap::new(),    // Empty storage
        })
    }

    /// Checks if an address corresponds to a contract account.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates how to distinguish between
    /// regular accounts (that hold value) and contract accounts (that hold code).
    /// This is a fundamental concept in blockchain systems.
    /// 
    /// NOTE: This is currently a simplified implementation that always returns true.
    /// In a real system, this would check if the account has code deployed.
    /// 
    /// PARAMETERS:
    /// - _addr: The address to check
    /// 
    /// RETURNS: true if the address is a contract, false otherwise
    pub fn is_contract(&self, _addr: Address) -> bool {
        // EDUCATIONAL: In a real implementation, this would check if the account has code
        // self.accounts.get(addr).map_or(false, |acc| acc.code.is_some())
        return true;
    }   

    /// Deploys a contract to a specific address.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates smart contract deployment.
    /// When a contract is deployed, it creates or updates an account with
    /// the contract's bytecode and marks it as a contract account.
    /// 
    /// DEPLOYMENT PROCESS:
    /// 1. Get or create the account at the specified address
    /// 2. Set the account's code to the provided bytecode
    /// 3. Mark the account as a contract
    /// 
    /// SECURITY: In real systems, contract deployment would include
    /// additional checks like code validation, gas limits, etc.
    /// 
    /// PARAMETERS:
    /// - addr: The address where the contract should be deployed
    /// - code: The bytecode of the contract to deploy
    pub fn deploy_contract(&mut self, addr: Address, code: Vec<u8>) {
        // EDUCATIONAL: Get or create the account at the specified address
        let acc = self.accounts.entry(addr).or_insert_with(|| Account {
            nonce: 0,                    // No transactions yet
            balance: 0,                  // No initial balance
            code: Vec::new(),            // No code initially
            is_contract: false,          // Not a contract initially
            storage: BTreeMap::new(),    // Empty storage
        });
        
        // EDUCATIONAL: Set the contract code and mark as contract
        acc.code = code;                 // Deploy the bytecode
        acc.is_contract = true;          // Mark as contract account
    }

    /// Prints a human-readable representation of the current state.
    /// 
    /// EDUCATIONAL PURPOSE: This demonstrates state inspection and debugging.
    /// Being able to visualize the blockchain state is crucial for development,
    /// testing, and understanding how transactions affect the system.
    /// 
    /// OUTPUT FORMAT: Shows each account with its:
    /// - Address (in hexadecimal)
    /// - Balance
    /// - Nonce (transaction count)
    /// - Contract status
    /// - Code size
    /// - Storage contents
    /// 
    /// USAGE: Useful for debugging, testing, and educational demonstrations.
    pub fn pretty_print(&self) {
        println!("--- State Dump ---");
        for (addr, acc) in &self.accounts {
            // EDUCATIONAL: Display account address in hexadecimal format
            println!("  🔑 Address: 0x{}", hex_encode(addr.0));
            
            // EDUCATIONAL: Display account metadata
            println!("      - Balance: {}", acc.balance);
            println!("      - Nonce: {}", acc.nonce);
            println!("      - Is contract?: {}", acc.is_contract);
            println!("      - Code size: {} bytes", acc.code.len());
            
            // EDUCATIONAL: Display storage contents
            println!("      - Storage:");
            for (key, value) in &acc.storage {
                // EDUCATIONAL: Convert storage values to hexadecimal for readability
                let value_hex: Vec<String> = value.iter().map(|b| format!("{:02x}", b)).collect();
                
                // Try to parse the key as a storage map key (address + domain)
                if let Some((address, domain)) = Self::parse_storage_map_key(key) {
                    println!("          Key: {}-{} | Value ({} bytes): {}", domain, address, value.len(), value_hex.join(" "));
                } else {
                    // Fall back to showing the raw key
                    println!("          Key: {:<20} | Value ({} bytes): {}", key, value.len(), value_hex.join(" "));
                }
            }
            println!();
        }
        println!("--------------------");
    }
    
    /// Parses a storage map key to extract address and domain components.
    /// 
    /// Storage map keys are formatted as: [address_bytes][domain]
    /// where address_bytes is 20 bytes and domain is like "-Balances"
    fn parse_storage_map_key(key: &str) -> Option<(String, String)> {
        // Check if the key is long enough to contain an address (20 bytes = 40 hex chars)
        if key.len() < 40 {
            return None;
        }
        
        // Try to parse the first 40 characters as a hex address
        if let Ok(address_bytes) = hex::decode(&key[..40]) {
            if address_bytes.len() == 20 {
                // Convert to proper address format
                let address = format!("0x{}", &key[..40]);
                
                // Parse the domain (remaining hex characters)
                let domain_hex = &key[40..];
                if let Ok(domain_bytes) = hex::decode(domain_hex) {
                    if let Ok(domain_str) = String::from_utf8(domain_bytes) {
                        return Some((address, domain_str));
                    }
                }
            }
        }
        
        None
    }
}
