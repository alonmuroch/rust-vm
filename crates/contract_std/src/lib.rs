#![no_std]

#[macro_export]
macro_rules! contract {
    (
        $(
            pub fn $name:ident ( $( $arg:ident : $typ:ty ),* ) -> $ret:ty $body:block
        )*
    ) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name( $( $arg : $typ ),* ) -> $ret $body
        )*

        // Emit _start as the true entry point
        #[unsafe(no_mangle)]
        pub extern "C" fn _start() -> ! {
            let mut sink = 0;
            $(
                {
                    let result = $crate::__dispatch_by_arity!($name, ($($typ),*));
                    unsafe {
                        core::ptr::write_volatile(&mut sink, result);
                    }
                }
            )*
            loop {}
        }

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    };
}

#[macro_export]
macro_rules! __dispatch_by_arity {
    ($f:ident, ()) => {
        unsafe { $f() }
    };
    ($f:ident, ($a1:ty)) => {
        unsafe { $f(core::default::Default::default()) }
    };
    ($f:ident, ($a1:ty, $a2:ty)) => {
        unsafe {
            $f(core::default::Default::default(), core::default::Default::default())
        }
    };
    ($f:ident, ($a1:ty, $a2:ty, $a3:ty)) => {
        unsafe {
            $f(
                core::default::Default::default(),
                core::default::Default::default(),
                core::default::Default::default()
            )
        }
    };
}
