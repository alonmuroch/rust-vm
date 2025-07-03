/// Represents a function call input for the VM router
pub struct HostFuncCall {
    pub selector: u8,
    pub args: Vec<u8>,
}

/// Encodes multiple function calls into a single buffer for the guest VM router.
pub fn encode_router_calls(calls: &[HostFuncCall]) -> Vec<u8> {
    let mut encoded = Vec::new();

    for call in calls {
        let len = call.args.len();
        assert!(len <= 255, "argument too long for 1-byte length field");

        encoded.push(call.selector);
        encoded.push(len as u8);
        encoded.extend_from_slice(&call.args);
    }

    encoded
}
