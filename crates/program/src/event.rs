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
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(C)]      
        pub struct $name {
            
            $(pub $field: $ty),*,
            pub id: [u8; 32],
        }

        impl $name {
            pub fn new($($field: $ty),*) -> Self {
                // id from name
                let mut id = [0u8; 32];
                let name = stringify!($name).as_bytes();
                let copy_len = core::cmp::min(name.len(), 32);
                id[..copy_len].copy_from_slice(&name[..copy_len]);

                Self {
                    $($field: $field),*,
                    id: id,
                }
            }
        }

        impl $name {
            pub fn to_bytes(&self) -> &[u8] {
                const MAX: usize = 256;
                static mut BUFFER: [u8; MAX] = [0; MAX];
                let mut offset = 0;

                unsafe {
                    // Copy ID field first
                    BUFFER[offset..offset+32].copy_from_slice(&self.id);
                    offset += 32;

                    // Copy all user-defined fields
                    $(
                        let ptr = &self.$field as *const $ty as *const u8;
                        let len = core::mem::size_of::<$ty>();
                        let bytes = core::slice::from_raw_parts(ptr, len);
                        BUFFER[offset..offset + len].copy_from_slice(bytes);
                        offset += len;
                    )*

                    &BUFFER[..offset]
                }
            }
        }
    };
}
