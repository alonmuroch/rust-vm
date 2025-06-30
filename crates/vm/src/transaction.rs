#[derive(Debug)]
pub struct Transaction {
    pub to: [u8; 20],       // recipient address
    pub from: [u8; 32],     // sender public key/address
    pub data: Vec<u8>,      // input data
    pub value: u64,         // amount/value sent
    pub nonce: u64,         // transaction nonce
}