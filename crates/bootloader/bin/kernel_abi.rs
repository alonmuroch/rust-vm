// Auto-generated ABI client code
// DO NOT EDIT - Generated from ABI

// Note: This code assumes the following imports in the parent file:
// use program::types::address::Address;
// use program::types::result::Result;
// use program::call::call;

/// Client for interacting with KernelContract contract
pub struct KernelContract {
    pub address: Address,
}

impl KernelContract {
    /// Create a new contract client
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Call the main entry point directly (no routing)
    pub fn call_main(
        &self,
        caller: &Address,
        data: &[u8],
    ) -> Option<Result> {
        // Direct call without router encoding
        call(caller, &self.address, data)
    }

}
