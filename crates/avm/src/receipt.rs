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

use compiler::{EventAbi, ParamType};

impl TransactionReceipt {
    pub fn print_events_pretty(&self, abi_registry: &Vec<EventAbi>, writer: &mut dyn fmt::Write) {
        if self.events.is_empty() {
            let _ = writeln!(writer, "No events in receipt.");
            return;
        }

        for event in &self.events {
            Self::pretty_print_event(event, abi_registry, writer);
        }
    }

    pub fn pretty_print_event(event: &[u8], abi_registry: &Vec<EventAbi>, writer: &mut dyn fmt::Write) {
        if event.len() < 32 {
            let _ = writeln!(writer, "Invalid event: too short");
            return;
        }

        let mut id = [0u8; 32];
        id.copy_from_slice(&event[..32]);
        let data = &event[32..];

        if let Some(abi) = abi_registry.iter().find(|abi| abi.id() == id) {
            let _ = writeln!(writer, "  {}: (", abi.name);
            let mut offset = 0;

            let _ = writeln!(writer, "        ID: 0x{}", hex::encode(id));
            for (i, param) in abi.inputs.iter().enumerate() {
                let val = if param.indexed {
                    "<indexed>".to_string()
                } else {
                    match param.kind {
                        ParamType::Address => {
                            if offset + 20 > data.len() {
                                let _ = writeln!(writer, "  {}: <invalid - data too short>", param.name);
                                break;
                            }
                            let bytes: &[u8] = &data[offset..offset + 20];
                            offset += 20;
                            format!("0x{}", hex::encode(bytes))
                        }
                        ParamType::Uint(256) | ParamType::Uint(32) => {
                            if offset + 4 > data.len() {
                                let _ = writeln!(writer, "  {}: <invalid - data too short>", param.name);
                                break;
                            }
                            let bytes = &data[offset..offset + 4];
                            offset += 4;
                            let raw = u32::from_le_bytes(bytes.try_into().unwrap());
                            format!("{}", raw)
                        }
                        ParamType::Bool => {
                            if offset + 1 > data.len() {
                                let _ = writeln!(writer, "  {}: <invalid - data too short>", param.name);
                                break;
                            }
                            let b = data[offset];
                            offset += 1;
                            format!("{}", b != 0)
                        }
                        ParamType::Bytes => {
                            if offset + 1 > data.len() {
                                let _ = writeln!(writer, "  {}: <invalid - data too short>", param.name);
                                break;
                            }
                            let len = data[offset] as usize;
                            offset += 1;
                            if offset + len > data.len() {
                                let _ = writeln!(writer, "  {}: <invalid - data too short>", param.name);
                                break;
                            }
                            let bytes = &data[offset..offset + len];
                            offset += len;
                            format!("0x{}", hex::encode(bytes))
                        }
                        _ => {
                            let _ = writeln!(writer, "  {}: <unimplemented type>", param.name);
                            break;
                        }
                    }
                };

                let comma = if i + 1 < abi.inputs.len() { "," } else { "" };
                let _ = writeln!(writer, "\t{}: {}{}", param.name, val, comma);
            }

            let _ = writeln!(writer, "  )");
        } else {
            let _ = writeln!(writer, "Unknown event: 0x{}", hex::encode(id));
        }
    }
}
