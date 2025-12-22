#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::mem::forget;
use core::slice;
use kernel::{BootInfo, Config, Task};
use program::{log, logf};
use types::transaction::{Transaction, TransactionBundle, TransactionType};

const SYSCALL_CREATE_ACCOUNT: u32 = 12;

#[allow(dead_code)]
static mut KERNEL_TASK: Option<Task> = None;

/// Kernel entrypoint. Receives:
/// - `bundle_ptr`/`bundle_len`: encoded `TransactionBundle` prepared by the bootloader.
/// - `state_ptr`/`state_len`: optional state blob (currently unused).
/// - `boot_info_ptr`: bootloader handoff with stack + page-table root info.
#[unsafe(no_mangle)]
pub extern "C" fn _start(
    bundle_ptr: *const u8,
    bundle_len: usize,
    _state_ptr: *const u8,
    _state_len: usize,
    boot_info_ptr: *const BootInfo,
) {
    // Copy args to locals before any syscalls (ecall clobbers a0).
    let bundle_ptr = bundle_ptr;
    let bundle_len = bundle_len;

    log!("kernel boot");
    logf!("bundle_len=%d", bundle_len as u32);

    if let Some(info) = unsafe { boot_info_ptr.as_ref() } {
        let task = Task::kernel(info.root_ppn, info.kstack_top);
        unsafe {
            KERNEL_TASK = Some(task);
        }
        logf!(
            "boot_info: root_ppn=0x%x kstack_top=0x%x mem_size=%d",
            info.root_ppn,
            info.kstack_top,
            info.memory_size
        );
    } else {
        log!("boot_info missing; kernel task not initialized");
    }

    let encoded_bundle = unsafe { slice::from_raw_parts(bundle_ptr, bundle_len) };

    if let Some(bundle) = TransactionBundle::decode(encoded_bundle) {
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

fn execute_transaction(tx: &Transaction) {
    match tx.tx_type {
        TransactionType::CreateAccount => create_account(tx),
        _ => log!("executing transaction"),
    }
}

fn create_account(tx: &Transaction) {
    let code_size = tx.data.len();
    let is_contract = code_size > 0;

    let msg = format!(
        "Tx creating account at address {}. Is contract: {}. Code size: {} bytes.",
        tx.to, is_contract, code_size
    );
    let msg_ref: &str = msg.as_str();
    log!(msg_ref);

    let max = Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT;
    if code_size > max {
        panic!(
            "❌ Code size ({}) exceeds CODE_SIZE_LIMIT ({} bytes)",
            code_size, max
        );
    }

    let addr_ptr = tx.to.0.as_ptr();
    let code_ptr = tx.data.as_ptr();
    let code_len = tx.data.len();

    #[cfg(target_arch = "riscv32")]
    unsafe {
        let mut result: u32;
        core::arch::asm!(
            "li a7, {create}",
            "mv a1, {addr}",
            "mv a2, {code_ptr}",
            "mv a3, {code_len}",
            "ecall",
            "mv {out}, a0",
            create = const SYSCALL_CREATE_ACCOUNT,
            addr = in(reg) addr_ptr,
            code_ptr = in(reg) code_ptr,
            code_len = in(reg) code_len,
            out = lateout(reg) result,
        );
        if result == 0 {
            log!("account created via syscall");
        } else {
            log!("account creation failed via syscall");
        }
    }
    #[cfg(not(target_arch = "riscv32"))]
    {
        log!("(host) account creation syscall skipped (not riscv32)");
    }
}

#[inline(never)]
fn halt() -> ! {
    unsafe { core::arch::asm!("ebreak") };
    loop {}
}
