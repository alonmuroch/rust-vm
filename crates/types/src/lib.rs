#![no_std]  

extern crate alloc;

pub mod address;
pub use address::{Address, ADDRESS_LEN};

pub mod result;
pub use result::Result;

// O module
pub mod o;
pub use o::*; // Allow `$crate::O` in macros

pub mod primitives;
pub use primitives::*; 

pub mod transaction;
pub use transaction::*;

pub mod boot;
pub use boot::BootInfo;

// used for serialization
pub trait SerializeField {
    /// Appends `self` into `buf` at `*offset`, advancing the offset.
    fn serialize_field(&self, buf: &mut [u8], offset: &mut usize);
}
