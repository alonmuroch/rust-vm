use core::fmt;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
