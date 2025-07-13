use core::mem::{size_of, MaybeUninit};
use crate::{require, types::O, types::address::Address};
use crate::logf;

/// Trait for types that can be used as storage keys in `StorageMap`.
pub trait StorageKey {
    /// Returns the raw byte representation of the key.
    fn as_storage_key(&self) -> &[u8];
}

impl StorageKey for Address {
    fn as_storage_key(&self) -> &[u8] {
        &self.0
    }
}

pub struct StorageMap;

impl StorageMap {
    pub fn get<V>(key: &[u8]) -> O<V>
    where
        V: Copy + Default,
    {
        require(key.len() <= 64, b"key too long");

        let mut full_key = [0u8; 64];
        full_key[..key.len()].copy_from_slice(key);

        unsafe {
            let mut value_ptr: u32;
            core::arch::asm!(
                "li a7, 1", // syscall_storage_read
                "ecall",
                in("t0") full_key.as_ptr(),
                in("t1") key.len(),
                out("t6") value_ptr,
            );

            if value_ptr == 0 {
                return O::None;
            }

            let len_bytes = core::slice::from_raw_parts(value_ptr as *const u8, 4);
            let value_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

            if value_len != size_of::<V>() {
                return O::None;
            }

            let data_ptr = (value_ptr + 4) as *const u8;
            let buf = core::slice::from_raw_parts(data_ptr, value_len);

            let mut val = MaybeUninit::<V>::uninit();
            core::ptr::copy_nonoverlapping(buf.as_ptr(), val.as_mut_ptr() as *mut u8, value_len);
            O::Some(val.assume_init())
        }
    }

    pub fn set<V>(key: &[u8], val: V)
    where
        V: Copy,
    {
        require(key.len() <= 64, b"key too long");

        let mut full_key = [0u8; 64];
        full_key[..key.len()].copy_from_slice(key);

        let val_bytes = unsafe {
            core::slice::from_raw_parts((&val as *const V) as *const u8, size_of::<V>())
        };

        unsafe {
            core::arch::asm!(
                "li a7, 2", // syscall_storage_write
                "ecall",
                in("t0") full_key.as_ptr(),
                in("t1") key.len(),
                in("t2") val_bytes.as_ptr(),
                in("t3") val_bytes.len(),
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}


#[macro_export]
macro_rules! Map {
    ($name:ident) => {
        pub struct $name;

        impl $name {
            pub fn get<K, V>(key: &K) -> $crate::types::O<V>
            where
                K: $crate::StorageKey,
                V: Copy + Default,
            {
                const PREFIX: &[u8] = concat!(stringify!($name), "/").as_bytes();
                const MAX_KEY_LEN: usize = 64;

                let key_bytes = key.as_storage_key();
                let total_len = PREFIX.len() + key_bytes.len();
                $crate::require(total_len <= MAX_KEY_LEN, b"key too long");

                let mut buf = [0u8; MAX_KEY_LEN];
                buf[..PREFIX.len()].copy_from_slice(PREFIX);
                buf[PREFIX.len()..total_len].copy_from_slice(key_bytes);

                $crate::StorageMap::get::<V>(&buf[..total_len])
            }

            pub fn set<K, V>(key: &K, val: V)
            where
                K: $crate::StorageKey,
                V: Copy,
            {
                const PREFIX: &[u8] = concat!(stringify!($name), "/").as_bytes();
                const MAX_KEY_LEN: usize = 64;
                
                let key_bytes = key.as_storage_key();
                let total_len = PREFIX.len() + key_bytes.len();
                $crate::require(total_len <= MAX_KEY_LEN, b"key too long");

                let mut buf = [0u8; MAX_KEY_LEN];
                buf[..PREFIX.len()].copy_from_slice(PREFIX);
                buf[PREFIX.len()..total_len].copy_from_slice(key_bytes);

                $crate::StorageMap::set::<V>(&buf[..total_len], val);
            }
        }
    };
}



