
#[macro_export]
macro_rules! entrypoint {
    ($func:path) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn entrypoint(
            pubkey_ptr: *const u8,
            input_ptr: *const u8,
            input_len: usize,
            result_ptr: *mut $crate::result::Result,
        ) {
            let pubkey = {
                let slice = core::slice::from_raw_parts(pubkey_ptr, 32);
                let mut array = [0u8; 32];
                array.copy_from_slice(slice);
                $crate::pubkey::Pubkey(array)
            };
            let input = {
                core::slice::from_raw_parts(input_ptr, input_len)
            };

            let result = $func(pubkey, input);
            core::ptr::write(result_ptr, result);

            // 🛑 Explicitly halt to avoid fallthrough return (which compiles to `ret`)
            unsafe { core::arch::asm!("ebreak") };
            loop {}
        }
    };
}
