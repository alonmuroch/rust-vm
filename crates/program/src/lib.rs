#![no_std]  

pub mod pubkey;
pub mod result;
pub use pubkey::Pubkey;
pub use result::Result;
pub use state::State;

#[macro_use] // enables macro use across the crate
pub mod entrypoint;
#[macro_use]
pub mod storage;

// Re-export so `$crate::Persistent` works in the macro
pub use storage::Persistent;