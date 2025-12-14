//! Simple in-memory pages for the OS boot/runtime layers.

mod memory_page;
mod stacked_memory;

pub use memory_page::MemoryPage;
pub use stacked_memory::StackedMemory;
