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
       // SAFETY: We assume the memory starting at `result_ptr` is valid and readable.
        let ptr = result_ptr as *const u8;
        if ptr.is_null() {
            return None;
        }

        unsafe {
            // Read the `success` byte
            let success = *ptr != 0;

            // Read the next 4 bytes as the error code
            let raw = core::slice::from_raw_parts(ptr.add(1), 4);
            let error_code = u32::from_le_bytes(raw.try_into().unwrap());

            Some(Result { success, error_code })
        }
    }

    #[cfg(not(target_arch = "riscv32"))]
    pub unsafe fn from_ptr(_result_ptr: u32) -> Option<Self> {
        panic!("not implemented")
    }
}