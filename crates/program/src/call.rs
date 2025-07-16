use types::address::Address;
use types::result::Result;

pub fn call(from: &Address, to: &Address, input_data: &[u8]) -> Option<Result> {
    unsafe {
        let mut result_ptr: u32;
        core::arch::asm!(
            "li a7, 5",        // syscall ID for call_contract
            "ecall",
            in("a1") to.0.as_ptr(),
            in("a2") from.0.as_ptr(),
            in("a3") input_data.as_ptr(),
            in("a4") input_data.len(),
            out("a0") result_ptr,
        );

        if result_ptr == 0 {
            return None;
        }

        Result::from_ptr(result_ptr)
    }
}