#![no_std]  

pub mod address;
pub use address::Address;

pub mod result;
pub use result::Result;

// O module
pub mod o;
pub use o::*; // Allow `$crate::O` in macros

pub mod primitives;
pub use primitives::*; 

// used for serialization
pub trait SerializeField {
    /// Appends `self` into `buf` at `*offset`, advancing the offset.
    fn serialize_field(&self, buf: &mut [u8], offset: &mut usize);
}