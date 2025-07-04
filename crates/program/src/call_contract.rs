use core::arch::asm;
use types::address::Address;

/// Syscall #5: Inter-program call
/// Inputs:
///   t0: pointer to `to` address
///   t1: pointer to `from` address
///   t2: pointer to input data
///   t3: input data length
/// Output:
///   t6: result code (u32)
pub fn call(to: &Address, from: &Address, input_data: &[u8]) -> u32 {
    let result: u32;
    unsafe {
        asm!(
            "li a7, 5",        // syscall ID for call_program
            "ecall",           // trigger syscall
            in("t0") to.0.as_ptr(),
            in("t1") from.0.as_ptr(),
            in("t2") input_data.as_ptr(),
            in("t3") input_data.len(),
            out("t6") result,  // syscall return value in t6
        );
    }
    result
}
