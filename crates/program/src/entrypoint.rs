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
                assert!(input_len <= 1024);
                core::slice::from_raw_parts(input_ptr, input_len)
            };

            let result = $func(pubkey, input);
            *result_ptr = result;
        }

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    };
}
