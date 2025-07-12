#![no_std]  

pub mod address;
pub mod result;

// O module
pub mod o;
pub use o::*; // Allow `$crate::O` in macros