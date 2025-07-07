#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Result {
    pub success: bool,
    pub error_code: u32,
}

impl Result {
    /// # Safety
    /// Caller must ensure `ptr` points to at least 5 valid bytes in memory.
    pub unsafe fn from_ptr(ptr: u32) -> Self {
        let error_code = {
            let raw = unsafe { core::slice::from_raw_parts(ptr as *const u8, 4) };
            u32::from_le_bytes(raw.try_into().unwrap())
        };

        let success = unsafe {
            let ptr = ptr as *const u32;
            *ptr.add(4) != 0
        };

        Self { success: success, error_code }
    }
}