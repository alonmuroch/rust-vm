#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Pubkey, Result, vm_panic};
use program::persist_struct;
use core::convert::TryInto;

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_USER: [u8; 5] = *b"user\0";

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_CONFIG: [u8; 7] = *b"config\0";

#[link_section = ".rodata"]
#[no_mangle]
pub static PERSIST_SESSION: [u8; 8] = *b"session\0";

// Struct 1: User profile
persist_struct!(User, PERSIST_USER, {
    id: u64,
    active: bool,
    level: u8,
});

// Struct 2: Config
persist_struct!(Config, PERSIST_CONFIG, {
    retries: u32,
    timeout_ms: u16,
});

// Struct 3: Session stats
persist_struct!(Session, PERSIST_SESSION, {
    tx_count: u64,
    last_error: u32,
    completed: bool,
});

fn my_vm_entry(_caller: Pubkey, _data: &[u8]) -> Result {
  // --- User ---
    let mut user = match User::load() {
        Some(u) => u,
        None => User { id: 1001, active: true, level: 3 },
    };

    user.level += 1;
    user.store();

    // ... later ...

    let reloaded_user = match User::load() {
        Some(u) => u,
        None => User { id: 1001, active: true, level: 3 },
    };
    if reloaded_user.level != 4 {
        vm_panic(b"User level must be 4");
    }

    // --- Config ---
    let mut config = match Config::load() {
        Some(c) => c,
        None => Config { retries: 2, timeout_ms: 3000 },
    };

    // ... later ...

    let reloaded_config = match Config::load() {
        Some(c) => c,
        None => Config { retries: 2, timeout_ms: 3000 },
    };

    // --- Session ---
    let mut session = match Session::load() {
        Some(s) => s,
        None => Session { tx_count: 0, last_error: 0, completed: false },
    };

    // ... later ...

    let reloaded_session = match Session::load() {
        Some(s) => s,
        None => Session { tx_count: 0, last_error: 0, completed: false },
    };

    // Compute error code
    let error_code = (reloaded_user.level as u32)
                   + (reloaded_config.timeout_ms as u32)
                   + (reloaded_session.tx_count as u32);

    Result {
        success: true,
        error_code,
    }
}

entrypoint!(my_vm_entry);
