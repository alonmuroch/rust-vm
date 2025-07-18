use alloc::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Account {
    pub nonce: u64,
    pub balance: u128,
    pub code: Vec<u8>,
    pub is_contract: bool,

    pub storage: BTreeMap<String, Vec<u8>>,
}