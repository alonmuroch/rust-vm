#[derive(Debug, Clone)]
pub struct EventAbi {
    pub name: String,
    pub inputs: Vec<EventParam>,
}

#[derive(Debug, Clone)]
pub struct EventParam {
    pub name: String,
    pub kind: ParamType,
    pub indexed: bool,
}

impl EventAbi {
    /// Returns the event ID: first 32 bytes of the event name, padded with zeros.
    pub fn id(&self) -> [u8; 32] {
        let name_bytes = self.name.as_bytes();
        let mut id = [0u8; 32];
        let len = name_bytes.len().min(32);
        id[..len].copy_from_slice(&name_bytes[..len]);
        id
    }
}

#[derive(Debug, Clone)]
pub enum ParamType {
    Address,
    Uint(usize), // bits
    Bool,
    Bytes,
    String,
    // Extend as needed
}
