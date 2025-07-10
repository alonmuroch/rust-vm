#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Result {
    pub success: bool,
    pub error_code: u32,
}

impl Result {
    /// # Safety
    /// Caller must ensure `ptr` points to at least 5 valid bytes in memory.
    #[cfg(target_arch = "riscv32")]
    pub fn from_ptr(result_ptr: u32) -> Option<Self> {
        if result_ptr == 0 {
            return None;
        }

        // SAFETY: We assume the syscall returns a valid 5-byte buffer:
        // - 1 byte at `result_ptr` for the success flag (bool),
        // - followed by 4 bytes for the error code in little-endian order.
        //
        // We manually read each byte to avoid issues with alignment or target-specific
        // panics in `core::slice::from_raw_parts`, which may validate internal alignment
        // assumptions that are not satisfied in the guest environment.
        let success = unsafe { *(result_ptr as *const u8) } != 0;

        let b1 = unsafe { *((result_ptr + 1) as *const u8) };
        let b2 = unsafe { *((result_ptr + 2) as *const u8) };
        let b3 = unsafe { *((result_ptr + 3) as *const u8) };
        let b4 = unsafe { *((result_ptr + 4) as *const u8) };

        let error_code = u32::from_le_bytes([b1, b2, b3, b4]);

        Some(Result { success, error_code })
    }

    #[cfg(not(target_arch = "riscv32"))]
    pub unsafe fn from_ptr(_result_ptr: u32) -> Option<Self> {
        panic!("not implemented")
    }
}