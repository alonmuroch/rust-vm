use crate::avm::AVM;
use types::address::Address;
use vm::host_interface::HostInterface;

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
    fn call_program(&mut self, from: [u8; 20], to: [u8; 20], input_data: Vec<u8>) -> (u32, usize) {
        unsafe {
            return (*self.avm_ptr).call_contract(Address(from), Address(to), input_data);
        }
    }

    fn fire_event(&mut self, event: Vec<u8>) {
        unsafe {
            // SAFETY: self.avm_ptr must point to a valid AVM that has access to the callee's memory
            let avm = &mut *self.avm_ptr;
            avm.context_stack
                .current_mut()
                .expect("must have current context")
                .events
                .push(event.clone());

            let hex_string: String = event
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<_>>()
                .join(" ");

            println!("[sys_fire_event] Event bytes (hex): {}", hex_string);
        }
    }

    fn read_memory_page(
        &mut self,
        page_index: usize,
        guest_ptr: u32,
        len: usize,
    ) -> Option<Vec<u8>> {
        unsafe {
            // SAFETY: self.avm_ptr must point to a valid AVM that has access to the callee's memory
            let avm = &*self.avm_ptr;

            let ee = avm
                .context_stack
                .get(page_index)
                .expect("missing execution context");
            let vm = ee.vm.borrow();
            let page_ref = vm.memory.as_ref();

            // Assume the callee's memory manager is accessible here
            let mem = page_ref.mem();

            let start = guest_ptr as usize;
            let end = start.checked_add(len)?;

            if end > mem.len() {
                return None; // Out of bounds
            }

            Some(mem[start..end].to_vec())
        }
    }

    fn transfer(&mut self, to: [u8; 20], value: u64) -> bool {
        unsafe {
            let avm = &mut *self.avm_ptr;
            let to_addr = Address(to);

            // Use the active execution context to determine the sender.
            let ctx = match avm.context_stack.current() {
                Some(c) => c,
                None => return false,
            };
            avm.apply_transfer(ctx.from, to_addr, value)
        }
    }

    fn balance(&mut self, addr: [u8; 20]) -> u128 {
        unsafe {
            let avm = &mut *self.avm_ptr;
            let account = avm.state.get_account(&Address(addr));
            account.map(|a| a.balance).unwrap_or(0)
        }
    }
}
