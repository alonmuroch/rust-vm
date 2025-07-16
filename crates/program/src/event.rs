#[macro_export]
macro_rules! fire_event {
    ($event:expr) => {{
        let event = $event; // take ownership
        let event_bytes = event.to_bytes();
        let ptr = event_bytes.as_ptr() as u32;
        let len = event_bytes.len() as u32;
        unsafe {
            core::arch::asm!(
                "li a7, 6",
                "ecall",
                in("a1") ptr,
                in("a2") len,
            );
        }
    }};
}

#[macro_export]
macro_rules! event {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[repr(C)]
        pub struct $name {
            $(pub $field: $ty),*
        }

        impl $name {
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }
        }

        impl $name {
            fn to_bytes(&self) -> &[u8] {
                // SAFETY: `Self` is repr(C) and POD
                let ptr = self as *const _ as *const u8;
                let len = core::mem::size_of::<Self>();
                unsafe { core::slice::from_raw_parts(ptr, len) }
            }
        }
    };
}
