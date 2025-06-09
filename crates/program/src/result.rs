#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Result {
    pub success: bool,
    pub error_code: u32,
}

impl Result {
    /// Write this Result into memory at the given pointer
    pub unsafe fn write_to_memory(&self, ptr: *mut u8) {
        // write error_code (u32) at offset 0
        core::ptr::write(ptr as *mut u32, self.error_code);
        // write success (bool as u8) at offset 4
        core::ptr::write(ptr.add(4) as *mut u8, self.success as u8);
    }
}