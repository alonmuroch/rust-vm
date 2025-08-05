use core::mem::{size_of, MaybeUninit};
use crate::{require, types::O, types::address::Address};
use crate::logf;

/// Trait for types that can be used as storage keys in `StorageMap`.
pub trait StorageKey {
    /// Fills the provided buffer with the hex-encoded key.
    /// Returns the length of the encoded key.
    fn as_storage_key(&self) -> &[u8];
}

impl StorageKey for Address {
    fn as_storage_key(&self) -> &[u8] {
        &self.0
    }
}


pub struct StorageMap;

impl StorageMap {
    pub fn get<V>(domain: &[u8], key: &[u8]) -> O<V>
    where
        V: Copy + Default,
    {
        require(key.len() <= 64, b"key too long");
        require(domain.len() <= 64, b"domain too long");

        let mut full_key = [0u8; 64];
        full_key[..key.len()].copy_from_slice(key);

        unsafe {
            let mut value_ptr: u32;
            core::arch::asm!(
                "li a7, 1", // syscall_storage_read
                "ecall",
                in("a1") domain.as_ptr(), // a1 - domain ptr
                in("a2") domain.len(), // a2 - domain len
                in("a3") full_key.as_ptr(), // a3 - key ptr
                in("a4") key.len(), // a4 - key len
                out("a0") value_ptr, // a0
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

    pub fn set<V>(domain: &[u8], key: &[u8], val: V)
    where
        V: Copy,
    {
        require(key.len() <= 64, b"key too long");
        require(domain.len() <= 64, b"domain too long");

        let mut full_key = [0u8; 64];
        full_key[..key.len()].copy_from_slice(key);

        let val_bytes = unsafe {
            core::slice::from_raw_parts((&val as *const V) as *const u8, size_of::<V>())
        };
        
        unsafe {
            core::arch::asm!(
                "li a7, 2", // syscall_storage_write
                "ecall",
                in("a1") domain.as_ptr(), // a1 - domain ptr
                in("a2") domain.len(), // a2 - domain len
                in("a3") full_key.as_ptr(), // a3 - key ptr
                in("a4") key.len(), // a4 - key len
                in("a5") val_bytes.as_ptr(), // a5 - value ptr
                in("a6") val_bytes.len(), // a6 - value len
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
            pub const DOMAIN_NAME: &'static str = stringify!($name);
            pub const DOMAIN_NAME_LEN: usize = stringify!($name).len();
            const MAX_KEY_LEN: usize = 64;

            fn build_key<K: $crate::StorageKey>(key: K, out: &mut [u8]) -> usize {
                let key_bytes = key.as_storage_key();
                let key_len = key_bytes.len();

                // 🛡 Copy key bytes into separate buffer to prevent aliasing
                let mut tmp = [0u8; 64];
                tmp[..key_len].copy_from_slice(key_bytes);

                // ✅ Write key to output buffer
                out[..key_len].copy_from_slice(&tmp[..key_len]);

                $crate::require(key_len <= Self::MAX_KEY_LEN, b"key too long");

                key_len
            }

            pub fn get<K, V>(key: K) -> $crate::types::O<V>
            where
                K: $crate::StorageKey,
                V: Copy + Default,
            {
                let mut buf = [0u8; Self::MAX_KEY_LEN];
                let total_len = Self::build_key(key, &mut buf);
                $crate::StorageMap::get::<V>(Self::DOMAIN_NAME.as_bytes(), &buf[..total_len])
            }

            pub fn set<K, V>(key: K, val: V)
            where
                K: $crate::StorageKey,
                V: Copy,
            {
                let mut buf = [0u8; Self::MAX_KEY_LEN];
                let total_len = Self::build_key(key, &mut buf);
                $crate::StorageMap::set::<V>(Self::DOMAIN_NAME.as_bytes(), &buf[..total_len], val);
            }
        }
    };
}





