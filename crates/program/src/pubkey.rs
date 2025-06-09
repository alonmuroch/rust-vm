#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Pubkey(pub [u8; 32]);

impl Pubkey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}
