/// Simple Sv32-style page table entry used by the software MMU.
///
/// Layout mirrors the RISC-V Sv32 PTE fields:
/// - `V` (valid) gate keeps the entry.
/// - `R/W/X` control read/write/execute access.
/// - `U` marks user visibility (we keep `G/A/D` out for now).
/// - `PPN` holds the physical page number.
///
/// For non-leaf entries, `next_l2` indexes an L2 table; for leaf entries,
/// `ppn` points at the mapped frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pte {
    /// Valid bit: entry is present.
    pub valid: bool,
    /// Read permission.
    pub read: bool,
    /// Write permission.
    pub write: bool,
    /// Execute permission.
    pub exec: bool,
    /// User visibility (false = supervisor/kernel only).
    pub user: bool,
    /// Physical page number for a leaf mapping.
    pub ppn: usize,
    /// Index of next-level L2 table for non-leaf entries.
    pub next_l2: Option<usize>,
}

impl Pte {
    pub fn is_leaf(&self) -> bool {
        self.valid && (self.read || self.write || self.exec)
    }
}
