

#[derive(Clone, Debug)]
pub struct Account {
    pub nonce: u64,
    pub balance: u128,
    pub code: Vec<u8>,
}