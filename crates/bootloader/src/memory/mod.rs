//! Simple in-memory pages for the OS boot/runtime layers.

mod memory;
mod pte;

pub use memory::Memory;
pub(crate) use pte::Pte;
