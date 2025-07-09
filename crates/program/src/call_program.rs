use types::address::Address;
use types::result::Result;
use crate::logf;
// use core::convert::TryInto;

/// Syscall #5: Inter-program call
/// Inputs:
///   t0: pointer to `to` address
///   t1: pointer to `from` address
///   t2: pointer to input data
///   t3: input data length
/// Output:
///   t6: result code (u32)
pub fn call(from: &Address, to: &Address, input_data: &[u8]) -> u32 {
    unsafe {
        let mut result_ptr: u32 = 0;
        core::arch::asm!(
            "li a7, 5",        // syscall ID for call_contract
            "ecall",
            in("t0") to.0.as_ptr(),
            in("t1") from.0.as_ptr(),
            in("t2") input_data.as_ptr(),
            in("t3") input_data.len(),
            out("t6") result_ptr,
        );

        // logf!(b"ptrrr %d", result_ptr);

        // if result_ptr == 0 {
        //     return None;
        // }

        // logf!(b"ptrrr2 %d", result_ptr);

        // return Some(Result::from_ptr(result_ptr));
        result_ptr
    }
}


