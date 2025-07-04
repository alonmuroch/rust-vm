use vm::host_interface::HostInterface;
use types::address::Address;
use crate::avm::AVM;

#[derive(Debug)]
pub struct HostShim {
    pub avm_ptr: *mut AVM, // raw pointer to the AVM
}

impl HostShim {
    pub fn new(avm: &mut AVM) -> Self {
        HostShim {
            avm_ptr: avm as *mut AVM,
        }
    }
}

impl<'a> HostInterface for HostShim {
    fn call_contract(&mut self, from: [u8; 20], to: [u8; 20], input_data: Vec<u8>) -> u32 {
        unsafe {
            (*self.avm_ptr).call_contract(Address::new(from), Address::new(to), input_data);
        }
        0
    }
}
