use types::address::Address;

#[derive(Debug)]
pub struct Transaction {
    pub to: Address,       // recipient address
    pub from: Address,     // sender public key/address
    pub data: Vec<u8>,      // input data
    pub value: u64,         // amount/value sent
    pub nonce: u64,         // transaction nonce
}