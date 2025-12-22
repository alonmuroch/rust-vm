//! Simple in-memory pages for the OS boot/runtime layers.

mod memory;
mod pte;

pub use memory::Memory;
pub use vm::memory::Perms;
pub(crate) use pte::Pte;
