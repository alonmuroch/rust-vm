use vm::host_interface::HostInterface;
use types::address::Address;
use crate::avm::AVM;

// HostShim is a lightweight adapter that allows a VM to call back into the AVM.
// It implements the HostInterface trait and holds a raw pointer to the AVM.
//
// We use a raw pointer (*mut AVM) instead of &'a mut AVM to avoid borrow checker conflicts.
// This is necessary because AVM::call_contract creates new VMs, which also require access
// to the AVM via HostInterface. If we used &'a mut AVM, we’d get lifetime or multiple mutable
// borrow errors due to recursive calls.
//
// By using *mut AVM:
// - We avoid tracked mutable borrows
// - We preserve safety by ensuring the pointer is only dereferenced during the call
// - We allow recursive VM execution without violating Rust's ownership model
//
// This approach is safe in our case because:
// - Each VM invocation gets its own HostShim
// - The pointer never escapes its VM or outlives AVM
// - We do not access AVM concurrently or from multiple threads
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
            (*self.avm_ptr).call_contract(Address(from), Address(to), input_data);
        }
        0
    }
}
