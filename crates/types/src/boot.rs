//! Boot-time handoff structures shared between bootloader and kernel.
//!
//! These types live in `types` so both sides agree on layout without
//! introducing circular dependencies.

/// Minimal boot information passed from the bootloader to the kernel.
///
/// Fields are kept simple and `#[repr(C)]` so the bootloader can write this
/// structure into guest memory and the kernel can read it back verbatim.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootInfo {
    /// Root page-table physical page number to load into satp.
    pub root_ppn: u32,
    /// Top of the kernel stack.
    pub kstack_top: u32,
    /// Total physical memory size in bytes.
    pub memory_size: u32,
    /// First free physical page number after bootloader allocations.
    pub next_free_ppn: u32,
}

impl BootInfo {
    pub const fn new(root_ppn: u32, kstack_top: u32, memory_size: u32, next_free_ppn: u32) -> Self {
        Self {
            root_ppn,
            kstack_top,
            memory_size,
            next_free_ppn,
        }
    }
}
