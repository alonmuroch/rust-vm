#![no_std]  

pub mod address;
pub use address::Address;

pub mod result;
pub use result::Result;

// O module
pub mod o;
pub use o::*; // Allow `$crate::O` in macros