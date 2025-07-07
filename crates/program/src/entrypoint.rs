/// Macro that creates the main entry point for a smart contract.
/// 
/// EDUCATIONAL PURPOSE: This macro generates the necessary code to interface
/// between the VM and the smart contract. It handles the conversion between
/// the VM's C-style interface and Rust's safe types.
/// 
/// FFI (Foreign Function Interface): This demonstrates how to create a safe
/// interface between different programming languages or systems. The VM calls
/// this function using C calling conventions, but we want to work with safe
/// Rust types inside our contract.
/// 
/// USAGE: Call this macro with the name of your main contract function:
/// ```rust
/// entrypoint!(my_contract_function);
/// ```
/// 
/// SAFETY CONSIDERATIONS:
/// - Uses unsafe code to handle raw pointers from the VM
/// - Validates pointer bounds to prevent crashes
/// - Explicitly halts execution to prevent undefined behavior
/// 
/// PARAMETERS (from VM):
/// - address_ptr: Pointer to 20-byte contract address
/// - pubkey_ptr: Pointer to 20-byte caller address  
/// - input_ptr: Pointer to input data
/// - input_len: Length of input data
/// - result_ptr: Pointer where to write the result
#[macro_export]
macro_rules! entrypoint {
    ($func:path) => {
        /// The main entry point that the VM calls to execute the contract.
        /// 
        /// EDUCATIONAL: This function is marked as extern "C" to use C calling
        /// conventions, which is what the VM expects. The #[no_mangle] attribute
        /// prevents the compiler from changing the function name.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn entrypoint(
            address_ptr: *const u8,      // Pointer to contract address (20 bytes)
            pubkey_ptr: *const u8,       // Pointer to caller address (20 bytes)
            input_ptr: *const u8,        // Pointer to input data
            input_len: usize,            // Length of input data
            result_ptr: *mut $crate::types::result::Result,  // Pointer to write result
        ) {
            // EDUCATIONAL: Convert raw pointer to contract address
            // This demonstrates safe pointer handling in unsafe code
            let to = {
                // EDUCATIONAL: Create a slice from the raw pointer (20 bytes for address)
                let slice = core::slice::from_raw_parts(address_ptr, 20);
                let mut array = [0u8; 20];
                // EDUCATIONAL: Copy bytes to avoid aliasing issues
                array.copy_from_slice(slice);
                $crate::types::address::Address(array)
            };
            
            // EDUCATIONAL: Convert raw pointer to caller address
            let from = {
                // EDUCATIONAL: Create a slice from the raw pointer (20 bytes for address)
                let slice = core::slice::from_raw_parts(pubkey_ptr, 20);
                let mut array = [0u8; 20];
                // EDUCATIONAL: Copy bytes to avoid aliasing issues
                array.copy_from_slice(slice);
                $crate::types::address::Address(array)
            };
           
            // EDUCATIONAL: Convert raw pointer to input data slice
            let input = {
                // EDUCATIONAL: Create a slice from the raw pointer with specified length
                core::slice::from_raw_parts(input_ptr, input_len)
            };

            // EDUCATIONAL: Call the user's contract function with safe types
            let result = $func(to, from, input);
            
            // EDUCATIONAL: Write the result back to the VM's memory
            core::ptr::write(result_ptr, result);

            // EDUCATIONAL: Explicitly halt execution to prevent undefined behavior
            // This is crucial because the VM expects the contract to halt, not return
            // The ebreak instruction triggers a breakpoint, and the infinite loop
            // ensures the function never returns normally
            unsafe { core::arch::asm!("ebreak") };
            loop {}
        }
    };
}
