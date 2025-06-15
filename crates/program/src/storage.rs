/// Trait for persistent structs
pub trait Persistent {
    fn load() -> Option<Self>
    where
        Self: Sized;

    fn store(&self);
}

/// Macro that defines persistent structs with embedded static key
#[macro_export]
macro_rules! persist_struct {
    ($name:ident, $key_ident:ident, {
        $($field:ident : $type:ty),* $(,)?
    }) => {
        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            $(pub $field: $type),*
        }

        impl $name {
            fn key_ptr() -> *const u8 {
                $key_ident.as_ptr()
            }

            fn key_len() -> usize {
                $key_ident.len()
            }
            pub fn as_bytes(&self) -> &[u8] {
                let ptr = self as *const _ as *const u8;
                let len = core::mem::size_of::<Self>();
                unsafe { core::slice::from_raw_parts(ptr, len) }
            }

            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != core::mem::size_of::<Self>() {
                    return None;
                }

                let mut val = core::mem::MaybeUninit::<Self>::uninit();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        val.as_mut_ptr() as *mut u8,
                        bytes.len(),
                    );
                    Some(val.assume_init())
                }
            }

            pub fn load() -> Option<Self> {
                <$name as $crate::Persistent>::load()
            }

            pub fn store(&self) {
                <$name as $crate::Persistent>::store(self)
            }
        }

        impl $crate::Persistent for $name {
            fn load() -> Option<Self> {
                unsafe {
                    let key_ptr = $name::key_ptr();
                    let key_len = $name::key_len();

                    if key_len == 0 {
                        $crate::vm_panic(
                            concat!("❌ persistent key for `", stringify!($name), "` is empty").as_bytes()
                        );
                    }

                    let mut value_ptr: u32;
                    core::arch::asm!(
                        "li a7, 1",  // syscall_storage_read
                        "ecall",
                        in("a0") key_ptr,
                        in("a1") key_len,
                        out("a2") value_ptr,
                    );

                    $crate::logf!("🔑 key_ptr = 0x{:08x}, len = {}", key_ptr as usize, key_len);

                    if value_ptr == 0 {
                        return None;
                    }

                    let len_bytes = core::slice::from_raw_parts(value_ptr as *const u8, 4);
                    let value_len = u32::from_le_bytes([
                        len_bytes[0],
                        len_bytes[1],
                        len_bytes[2],
                        len_bytes[3],
                    ]) as usize;

                    let data_ptr = (value_ptr + 4) as *const u8;
                    let value_buf = core::slice::from_raw_parts(data_ptr, value_len);

                    Self::from_bytes(value_buf)
                }
            }

            
            // fn load() -> Option<Self> {
            //     unsafe {
            //         let key_ptr = $name::key_ptr();
            //         let key_len = $name::key_len();

            //         if key_len == 0 {
            //             $crate::vm_panic(
            //                 concat!("❌ persistent key for `", stringify!($name), "` is empty").as_bytes()
            //             );
            //         }

            //         let mut value_ptr: u32;
            //         core::arch::asm!(
            //             "li a7, 1",  // syscall_storage_read
            //             "ecall",
            //             in("a0") key_ptr,
            //             in("a1") key_len,
            //             out("a2") value_ptr,
            //         );

            //         $crate::logf!("🔑 key_ptr = 0x{:08x}, len = {}", key_ptr as usize, key_len);
            //         $crate::logf!("📥 value_ptr = 0x{:08x}", value_ptr);

            //         if value_ptr == 0 {
            //             return None;
            //         }

            //         let len_bytes = core::slice::from_raw_parts(value_ptr as *const u8, 4);
            //         let value_len = u32::from_le_bytes([
            //             len_bytes[0],
            //             len_bytes[1],
            //             len_bytes[2],
            //             len_bytes[3],
            //         ]) as usize;

            //         let data_ptr = (value_ptr + 4) as *const u8;
            //         $crate::logf!("📦 data_ptr = 0x{:08x}, len = {}", data_ptr as usize, value_len);

            //         // 🧩 Optional: dump raw bytes as hex
            //         {
            //             let raw_data = core::slice::from_raw_parts(data_ptr, value_len);
            //             let mut hex_buf = [0u8; 128];

            //             let hex_len = match $crate::hex::encode_to_slice(raw_data, &mut hex_buf) {
            //                 Ok(()) => raw_data.len() * 2,
            //                 Err(_) => 0,
            //             };

            //             let prefix = b"\xf0\x9f\x93\xa6 raw = 0x";
            //             let mut final_msg = [0u8; 160];

            //             final_msg[..prefix.len()].copy_from_slice(prefix);
            //             final_msg[prefix.len()..prefix.len() + hex_len]
            //                 .copy_from_slice(&hex_buf[..hex_len]);

            //             $crate::log::vm_log(&final_msg[..prefix.len() + hex_len]);
            //         }

            //         Self::from_bytes(core::slice::from_raw_parts(data_ptr, value_len))
            //     }
            // }



            fn store(&self) {
                unsafe {
                    let key_ptr = $name::key_ptr();
                    let key_len = $name::key_len();

                    if key_len == 0 {
                        $crate::vm_panic(
                            concat!("❌ persistent key for `", stringify!($name), "` is empty").as_bytes()
                        );
                    }

                    let val_buf = self.as_bytes();

                    let mut buf: [u8; core::mem::size_of::<Self>()] = core::mem::zeroed();
                    let len = buf.len();
                    core::ptr::copy_nonoverlapping(val_buf.as_ptr(), buf.as_mut_ptr(), len);

                    let val_ptr = buf.as_ptr();
                    let val_len = len;

                    core::arch::asm!(
                        "li a7, 2", // syscall_storage_write
                        "ecall",
                        in("a0") key_ptr,
                        in("a1") key_len,
                        in("a2") val_ptr,
                        in("a3") val_len,
                        options(readonly, nostack, preserves_flags)
                    );
                }
            }
        }
    };
}
