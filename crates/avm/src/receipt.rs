use types::{Result}; 
use crate::transaction::Transaction;

/// Represents the result of a transaction execution.
#[derive(Debug, Clone)]
pub struct TransactionReceipt {
    /// Hash of the transaction.
    pub tx: Transaction,

    /// Cumulative gas used in the block including this transaction.
    // pub cumulative_gas_used: u64,

    // /// Gas used by this transaction alone.
    // pub gas_used: u64,

    pub result: Result,

    /// List of log entries generated during execution.
    pub events: Vec<Vec<u8>>,
}

impl TransactionReceipt {
    /// Creates a new TransactionReceipt.
    pub fn new(tx: Transaction, result: Result) -> Self {
        TransactionReceipt {
            tx,
            // cumulative_gas_used: 0,
            // gas_used: 0,
            result,
            events: Vec::new(),
        }
    }

    /// Adds an event to the receipt.
    pub fn add_event(&mut self, event: Vec<u8>) -> &TransactionReceipt {
        self.events.push(event);
        self
    }
    
    /// Optionally add multiple events at once.
    pub fn set_events(mut self, events: Vec<Vec<u8>>) -> Self {
        self.events = events;
        self
    }
}

use core::fmt;

impl fmt::Display for TransactionReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Transaction Receipt ===")?;
        writeln!(f, "From: {:?}", self.tx.from)?;
        writeln!(f, "To: {:?}", self.tx.to)?;
        writeln!(f, "Result: {:?}", self.result)?;
        writeln!(f, "Events:")?;

        for (i, event) in self.events.iter().enumerate() {
            let hex = event.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            writeln!(f, "  [{}] {}", i, hex)?;
        }

        Ok(())
    }
}
