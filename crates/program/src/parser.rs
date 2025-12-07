//! Simple parser for reading typed values from a byte slice.
use core::convert::TryInto;
use types::address::Address;
use crate::vm_panic;

pub struct DataParser<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DataParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn ensure(&self, bytes: usize) {
        if self.offset + bytes > self.data.len() {
            vm_panic(b"insufficient input data");
        }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    pub fn read_bytes(&mut self, len: usize) -> &'a [u8] {
        self.ensure(len);
        let start = self.offset;
        self.offset += len;
        &self.data[start..start + len]
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes: [u8; 4] = self.read_bytes(4).try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    pub fn read_u64(&mut self) -> u64 {
        let bytes: [u8; 8] = self.read_bytes(8).try_into().unwrap();
        u64::from_le_bytes(bytes)
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_bytes(1)[0] != 0
    }

    pub fn read_address(&mut self) -> Address {
        let bytes = self.read_bytes(20);
        let mut arr = [0u8; 20];
        arr.copy_from_slice(bytes);
        Address(arr)
    }
}
