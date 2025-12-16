#![no_std]
#![no_main]

extern crate alloc;

#[cfg(target_arch = "riscv32")]
mod allocator;
use program::log;
use program::logf;

use core::slice;
use core::mem::forget;
use types::transaction::{Transaction, TransactionBundle};

#[cfg(target_arch = "riscv32")]
#[global_allocator]
static ALLOC: allocator::VmAllocator = allocator::VmAllocator;

/// Kernel entrypoint. Receives a pointer/length pair to an encoded `TransactionBundle`
/// (produced by the bootloader) and walks each transaction.
#[unsafe(no_mangle)]
pub extern "C" fn _start(bundle_ptr: *const u8, bundle_len: usize) {
    // Copy args to locals before any syscalls (ecall clobbers a0).
    let ptr = bundle_ptr;
    let len = bundle_len;

    log!("kernel boot");
    logf!("bundle_len=%d", len as u32);

    let encoded = unsafe { slice::from_raw_parts(ptr, len) };

    if let Some(bundle) = TransactionBundle::decode(encoded) {
        let count = bundle.transactions.len();
        logf!("decoded tx count=%d", count as u32);
        for i in 0..count {
            logf!("processing tx %d/%d", (i + 1) as u32, count as u32);
            if let Some(tx) = bundle.transactions.get(i) {
                execute_transaction(tx);
            } else {
                logf!("missing tx at index %d", i as u32);
            }
        }
        // Avoid drop-time teardown that can allocate/deallocate; we halt immediately.
        forget(bundle);
    } else {
        log!("bundle decode failed");
    }
    log!("finished bundle execution");
    halt();
}

fn execute_transaction(_tx: &Transaction) {
    log!("executing transaction");
}

#[inline(never)]
fn halt() -> ! {
    // Signal completion to the host by triggering a trap and stop execution.
    unsafe { core::arch::asm!("ebreak") };
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "riscv32")]
    {
        let msg_bytes = if let Some(s) = info.message().as_str() {
            s.as_bytes()
        } else {
            b"kernel panic (non-str message)"
        };
        unsafe {
            core::arch::asm!(
                "li a7, 3", // SYSCALL_PANIC
                "ecall",
                in("a0") msg_bytes.as_ptr(),
                in("a1") msg_bytes.len(),
            );
            core::arch::asm!("ebreak", options(nomem, nostack));
        }
        loop {}
    }

    #[cfg(not(target_arch = "riscv32"))]
    {
        panic!("kernel panic: {:?}", info);
    }
}
