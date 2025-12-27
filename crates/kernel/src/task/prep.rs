use crate::{AddressSpace, Config, Task, mmu};
use program::{log, logf};
use types::address::Address;

use super::{
    alloc_asid, FROM_PTR_ADDR, HEAP_BYTES, INPUT_BASE_ADDR, PAGE_SIZE, PROGRAM_VA_BASE,
    PROGRAM_WINDOW_BYTES, REG_A0, REG_A1, REG_A2, REG_A3, REG_SP, STACK_BYTES,
    TO_PTR_ADDR, TRAMPOLINE_CODE, TRAMPOLINE_VA,
};

/// Create a new task for a program and map its virtual address window via syscalls.
///
/// This sets up:
/// - Maps a fixed VA window [PROGRAM_VA_BASE, PROGRAM_VA_BASE + PROGRAM_WINDOW_BYTES).
/// - Returns a Task with the new address space.
///
/// The caller is responsible for copying program bytes into the mapped window
/// and initializing the user trapframe (PC/SP/args) before running.
pub fn prep_program_task(
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
        AddressSpace::new(
            root_ppn,
            asid,
            PROGRAM_VA_BASE,
            PROGRAM_WINDOW_BYTES as u32,
        ),
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
