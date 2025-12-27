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
// prep_program_task(kstack_top, to, from, code, input, entry_off):
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

use crate::{AddressSpace, Config, Task, mmu};
use crate::global::{BOOT_INFO, NEXT_ASID, TASKS};
use program::log;
use program::logf;
use types::{address::Address, ADDRESS_LEN};

const PAGE_SIZE: usize = 4096;
const STACK_BYTES: usize = 0x4000; // 16 KiB user stack
const HEAP_BYTES: usize = 0x8000; // 32 KiB user heap
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

/// Create a new task for a program and map its virtual address window via syscalls.
///
/// This sets up:
/// - Maps a fixed VA window [PROGRAM_VA_BASE, PROGRAM_VA_BASE + PROGRAM_WINDOW_BYTES).
/// - Returns a Task with the new address space and provided kernel stack top.
///
/// The caller is responsible for copying program bytes into the mapped window
/// and initializing the user trapframe (PC/SP/args) before running.
pub fn prep_program_task(
    kstack_top: u32,
    to: &Address,
    from: &Address,
    code: &[u8],
    input: &[u8],
    entry_off: u32,
) -> Option<Task> {
    if input.len() > Config::MAX_INPUT_LEN {
        log!("launch_program: input too large");
        return None;
    }

    // Make sure the kernel page allocator does not hand out frames that the
    // kernel heap already consumed (account code lives there). We conservatively
    // push the allocator toward the top of memory so new user roots/tables
    // don't overlap heap-backed data.
    let total_ppn = unsafe {
        BOOT_INFO
            .get_mut()
            .as_ref()
            .map(|b| b.memory_size / PAGE_SIZE as u32)
            .unwrap_or(0)
    };
    let reserve = (PROGRAM_WINDOW_BYTES / PAGE_SIZE) as u32 + 4; // user window + a few tables
    if total_ppn > reserve {
        let min_ppn = total_ppn - reserve;
        mmu::bump_page_allocator(min_ppn);
        logf!(
            "prep_program_task: bump page alloc to ppn=0x%x (total_ppn=0x%x)",
            min_ppn,
            total_ppn
        );
    }

    let asid = alloc_asid();
    let root_ppn = match mmu::alloc_root() {
        Some(ppn) => ppn,
        None => {
            logf!("launch_program: no free root PPN available");
            return None;
        }
    };
    logf!(
        "prep_program_task: new root=0x%x asid=%d code_len=%d input_len=%d entry_off=0x%x",
        root_ppn,
        asid as u32,
        code.len() as u32,
        input.len() as u32,
        entry_off,
    );
    let window_end = PROGRAM_VA_BASE.wrapping_add(PROGRAM_WINDOW_BYTES as u32);
    logf!(
        "launch_program: asid=%d root=0x%x map=[0x%x,0x%x)",
        asid as u32,
        root_ppn,
        PROGRAM_VA_BASE,
        window_end
    );
    let perms = mmu::PagePerms::user_rwx();
    if !mmu::map_user_range_for_root(root_ppn, PROGRAM_VA_BASE, PROGRAM_WINDOW_BYTES, perms) {
        logf!("launch_program: mapping failed (root=0x%x)", root_ppn);
        return None;
    }

    // Copy the full program image starting at VA 0 so section offsets (e.g. .text at 0x400)
    // land where the ELF expected them. Entry offset is provided by the caller.
    if entry_off as usize >= code.len() {
        logf!(
            "prep_program_task: entry_off 0x%x is outside code len %d",
            entry_off,
            code.len() as u32
        );
        return None;
    }
    if code.len() >= entry_off as usize + 8 {
        let head = u32::from_le_bytes([
            code[entry_off as usize],
            code[entry_off as usize + 1],
            code[entry_off as usize + 2],
            code[entry_off as usize + 3],
        ]);
        let head2 = u32::from_le_bytes([
            code[entry_off as usize + 4],
            code[entry_off as usize + 5],
            code[entry_off as usize + 6],
            code[entry_off as usize + 7],
        ]);
        logf!(
            "prep_program_task: code head[0..8]=0x%x 0x%x (entry_off=0x%x)",
            head,
            head2,
            entry_off,
        );
    }
    let nz_count = code.iter().filter(|&&b| b != 0).count();
    let local_first_nz = code.iter().position(|&b| b != 0).unwrap_or(code.len());
    logf!(
        "prep_program_task: local code stats first_nz=0x%x nz_count=%d",
        local_first_nz as u32,
        nz_count as u32
    );
    if !mmu::copy_into_user(root_ppn, PROGRAM_VA_BASE, code) {
        logf!("launch_program: failed to copy code into root=0x%x", root_ppn);
        return None;
    }
    logf!(
        "prep_program_task: copied code to 0x%x len=%d nz_count=%d",
        PROGRAM_VA_BASE,
        code.len() as u32,
        nz_count as u32
    );
    if !mmu::copy_into_user(root_ppn, TO_PTR_ADDR, &to.0) {
        logf!("launch_program: failed to copy 'to' address into root=0x%x", root_ppn);
        return None;
    }
    if !mmu::copy_into_user(root_ppn, FROM_PTR_ADDR, &from.0) {
        logf!("launch_program: failed to copy 'from' address into root=0x%x", root_ppn);
        return None;
    }
    if !mmu::copy_into_user(root_ppn, INPUT_BASE_ADDR, input) {
        logf!("launch_program: failed to copy input into root=0x%x", root_ppn);
        return None;
    }
    logf!(
        "prep_program_task: copied args to=0x%x from=0x%x input=0x%x len=%d",
        TO_PTR_ADDR,
        FROM_PTR_ADDR,
        INPUT_BASE_ADDR,
        input.len() as u32
    );

    // Sanity check where the code landed in the user root.
    let entry_va = PROGRAM_VA_BASE.wrapping_add(entry_off);
    let user_phys = mmu::translate_user_va(root_ppn, entry_va).unwrap_or(usize::MAX);
    let user_word = mmu::peek_word(root_ppn, entry_va).unwrap_or(0);
    logf!(
        "prep_program_task: code VA=0x%x user_phys=0x%x user_word=0x%x code_start=0x%x",
        entry_va,
        user_phys as u32,
        user_word,
        entry_off
    );
    // Install a small trampoline page mapped in both roots so we can switch
    // satp safely before jumping into the user program.
    let tramp_perms = mmu::PagePerms::user_rwx();
    if !mmu::map_user_range_for_root(root_ppn, TRAMPOLINE_VA, PAGE_SIZE, tramp_perms) {
        logf!("prep_program_task: failed to map trampoline page in user root");
        return None;
    }
    let tramp_phys = match mmu::translate_user_va(root_ppn, TRAMPOLINE_VA) {
        Some(p) => p as u32,
        None => {
            log!("prep_program_task: trampoline VA not mapped");
            return None;
        }
    };
    if !mmu::map_physical_range_for_root(
        mmu::current_root(),
        TRAMPOLINE_VA,
        tramp_phys,
        PAGE_SIZE,
        tramp_perms,
    ) {
        log!("prep_program_task: failed to mirror trampoline into kernel root");
        return None;
    }
    let mut tramp_bytes = [0u8; TRAMPOLINE_CODE.len() * 4];
    for (i, word) in TRAMPOLINE_CODE.iter().enumerate() {
        tramp_bytes[i * 4..(i + 1) * 4].copy_from_slice(&word.to_le_bytes());
    }
    if !mmu::copy_into_user(root_ppn, TRAMPOLINE_VA, &tramp_bytes) {
        log!("prep_program_task: failed to populate trampoline code");
        return None;
    }
    let tramp_phys_2 = mmu::translate_user_va(root_ppn, TRAMPOLINE_VA + 4).unwrap_or(usize::MAX);
    logf!(
        "prep_program_task: trampoline mapped va=0x%x phys=0x%x phys(+4)=0x%x",
        TRAMPOLINE_VA,
        tramp_phys,
        tramp_phys_2 as u32
    );

    let mut task = Task::new(
        AddressSpace::new(root_ppn, asid),
        kstack_top,
        Config::HEAP_START_ADDR as u32,
    );
    // Set up initial trapframe.
    let stack_top = PROGRAM_VA_BASE
        .wrapping_add((Config::CODE_SIZE_LIMIT + Config::RO_DATA_SIZE_LIMIT + STACK_BYTES) as u32);
    task.tf.pc = entry_va;
    task.tf.regs[REG_SP] = stack_top;
    task.tf.regs[REG_A0] = TO_PTR_ADDR;
    task.tf.regs[REG_A1] = FROM_PTR_ADDR;
    task.tf.regs[REG_A2] = INPUT_BASE_ADDR;
    task.tf.regs[REG_A3] = input.len() as u32;
    logf!(
        "prep_program_task: trapframe pc=0x%x sp=0x%x a0=0x%x a1=0x%x a2=0x%x a3=%d",
        task.tf.pc,
        task.tf.regs[REG_SP],
        task.tf.regs[REG_A0],
        task.tf.regs[REG_A1],
        task.tf.regs[REG_A2],
        task.tf.regs[REG_A3],
    );
    // Also log the expected user stack window for sanity.
    let stack_base = stack_top.saturating_sub(STACK_BYTES as u32);
    logf!(
        "prep_program_task: stack window=[0x%x,0x%x) heap_base=0x%x",
        stack_base,
        stack_top,
        Config::HEAP_START_ADDR as u32
    );

    Some(task)
}

/// One-way context switch into a user task:
/// - Saves the current kernel frame into TASKS[0]
/// - Loads the task's satp/regs/pc and jumps to user code (no return path yet)
pub fn run_task(task: &Task) {
    let kernel_root = mmu::current_root();
    let target_root = task.addr_space.root_ppn;
    logf!(
        "run_task: switching satp 0x%x -> 0x%x asid=%d pc=0x%x sp=0x%x",
        kernel_root,
        target_root,
        task.addr_space.asid as u32,
        task.tf.pc,
        task.tf.regs[REG_SP],
    );
    // Save the current kernel frame (SP/RA/PC) into the kernel task slot (index 0).
    let mut saved_sp: u32;
    let mut saved_ra: u32;
    let mut saved_pc: u32;
    unsafe {
        core::arch::asm!("mv {out}, sp", out = out(reg) saved_sp);
        core::arch::asm!("mv {out}, ra", out = out(reg) saved_ra);
        core::arch::asm!("auipc {out}, 0", out = out(reg) saved_pc);
    }
    logf!(
        "run_task: saved kernel frame sp=0x%x ra=0x%x pc=0x%x",
        saved_sp,
        saved_ra,
        saved_pc
    );
    // Stash the kernel context so a future return path could restore it.
    unsafe {
        if let Some(tasks) = TASKS.get_mut() {
            if let Some(kernel_task) = tasks.get_mut(0) {
                kernel_task.addr_space.root_ppn = kernel_root;
                kernel_task.tf.regs[REG_SP] = saved_sp;
                kernel_task.tf.regs[REG_RA] = saved_ra;
                kernel_task.tf.pc = saved_pc;
            }
        }
    }
    // Update the helper's view of the current root before switching.
    mmu::set_current_root(target_root);
    // Set up registers and jump to the shared trampoline page (mapped in both
    // the kernel and user roots). The trampoline will write satp and transfer
    // control to the user PC.
    unsafe {
        core::arch::asm!(
            "mv t0, {satp}",   // satp to write
            "mv t1, {pc}",     // user PC
            "mv ra, zero",
            "mv sp, {sp}",
            "mv a0, {a0}",
            "mv a1, {a1}",
            "mv a2, {a2}",
            "mv a3, {a3}",
            "jr {tramp}",
            satp = in(reg) target_root,
            pc = in(reg) task.tf.pc,
            sp = in(reg) task.tf.regs[REG_SP],
            a0 = in(reg) task.tf.regs[REG_A0],
            a1 = in(reg) task.tf.regs[REG_A1],
            a2 = in(reg) task.tf.regs[REG_A2],
            a3 = in(reg) task.tf.regs[REG_A3],
            tramp = in(reg) TRAMPOLINE_VA,
            options(noreturn)
        );
    }
}

fn alloc_asid() -> u16 {
    unsafe {
        let counter = NEXT_ASID.get_mut();
        let asid = if *counter == 0 { 1 } else { *counter };
        *counter = asid.wrapping_add(1);
        asid
    }
}
