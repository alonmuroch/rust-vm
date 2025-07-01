/// Represents a 20-byte address (e.g., Ethereum-style account or contract).
pub type Address = [u8; 20];

/// Represents a single execution context during contract calls.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The address that initiated the current call.
    pub from: Address,

    /// The address currently receiving the call.
    pub to: Address,
}
impl ExecutionContext {
    pub fn new(from: Address, to: Address) -> Self {
        Self { from, to }
    }
}