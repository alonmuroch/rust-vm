#![allow(dead_code)]

// Program launch flow (kernel side)
// ---------------------------------
// Goals:
// - Create a fresh address space for each program call (new root PPN + ASID).
// - Map a fixed, contiguous user window starting at VA 0x0 that holds:
//     * Code/rodata (program bytes copied starting at VA 0x0; entry at `entry_off`)
//     * A user stack (STACK_BYTES)
//     * A user heap (HEAP_BYTES) with input placed at INPUT_BASE_ADDR
// - Copy call arguments (to/from addresses + input buffer) into that window.
// - Prepare a trapframe with PC/SP/args and transfer control to user code.
//
// Key pieces:
// - PROGRAM_WINDOW_BYTES covers code + rodata + stack + heap: a single map call per program.
// - TRAMPOLINE_VA is one page immediately after the user window, mapped into both
//   the kernel root and the new user root. It contains two instructions:
//     csrw satp, t0
//     jr   t1
//   This lets us switch satp safely from a VA that stays valid across the root change.
//
// prep_program_task(to, from, code, input, entry_off):
// 1) Allocate ASID and a fresh root PPN; map the user window with user_rwx perms.
// 2) Copy program code starting at VA 0 (so section offsets are preserved), copy args (to/from/input).
// 3) Map the trampoline page into the user root and mirror the same physical page
//    into the current kernel root; write TRAMPOLINE_CODE into it.
// 4) Build a Task with AddressSpace {root_ppn, asid} and set trapframe:
//       pc = PROGRAM_VA_BASE + entry_off
//       sp = top of user stack within the window
//       a0..a3 = to/from/input_base/input_len
//    Caller can push the task into TASKS for bookkeeping.
//
// run_task(task):
// - Save the current kernel frame (sp/ra/pc) into TASKS[0] for a future return path.
// - Preload t0 with the task root (satp value) and t1 with the user PC; load
//   user sp and a0..a3; clear ra.
// - jr TRAMPOLINE_VA. The trampoline executes under the old root, writes satp
//   to the new root, and immediately jr t1 into user code. There is no return
//   path yet; this is a one-way handoff.
//
// Notes:
// - The window and trampoline VAs are low for simplicity; nothing here relocates.
// - We currently do not touch sstatus/mstatus or perform sfence.vma; add those
//   when modeling fuller privilege transitions.

use crate::Config;
use crate::global::NEXT_ASID;
use types::ADDRESS_LEN;

pub mod task;
pub mod prep;
pub mod run;

pub use task::{AddressSpace, Task, TrapFrame};
pub use prep::prep_program_task;
pub use run::run_task;

const PAGE_SIZE: usize = 4096;
const STACK_BYTES: usize = 0x4000; // 16 KiB user stack
pub const HEAP_BYTES: usize = 0x8000; // 32 KiB user heap
pub const PROGRAM_VA_BASE: u32 = 0x0;
// Location of the page that hosts the satp-switch trampoline. Kept just past
// the user window so it does not collide with program text/stack/heap. This VA
// is mapped into both roots so the satp write does not invalidate the
// instruction stream mid-flight.
const TRAMPOLINE_VA: u32 = (PROGRAM_VA_BASE as usize + PROGRAM_WINDOW_BYTES) as u32;
const fn align_up(val: usize, align: usize) -> usize {
    (val + (align - 1)) & !(align - 1)
}

/// Total mapped window for a program: code/rodata, stack, and heap.
pub const PROGRAM_WINDOW_BYTES: usize = align_up(
    Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT + STACK_BYTES + HEAP_BYTES,
    PAGE_SIZE,
);

const REG_SP: usize = 2;
const REG_RA: usize = 1;
const REG_A0: usize = 10;
const REG_A1: usize = 11;
const REG_A2: usize = 12;
const REG_A3: usize = 13;
// Raw RISC-V words for the trampoline used to switch satp safely while
// executing from a page mapped in both the kernel and user roots. The kernel
// loads t0 = target satp and t1 = user PC before entering this stub so we can
// change roots and immediately branch to user code without returning to
// unmapped kernel text.
// t0: target satp value, t1: user PC (jump target).
const TRAMPOLINE_CODE: [u32; 2] = [
    0x1802_9073, // csrw satp, t0
    0x0003_0067, // jr t1
];

const TO_PTR_ADDR: u32 = 0x120;
const FROM_PTR_ADDR: u32 = TO_PTR_ADDR + ADDRESS_LEN as u32;
const INPUT_BASE_ADDR: u32 = Config::HEAP_START_ADDR as u32;

pub(super) fn alloc_asid() -> u16 {
    unsafe {
        let counter = NEXT_ASID.get_mut();
        let asid = if *counter == 0 { 1 } else { *counter };
        *counter = asid.wrapping_add(1);
        asid
    }
}
