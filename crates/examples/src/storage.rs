#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, Result, require};
use program::types::address::Address; 
use program::persist_struct;

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
    retries: u64,
    timeout_ms: u64,
});

// Struct 3: Session stats
persist_struct!(Session, PERSIST_SESSION, {
    tx_count: u64,
    last_error: u64,
    completed: bool,
});

fn my_vm_entry(_self_address: Address, _caller: Address, _data: &[u8]) -> Result {
    // --- User ---
    let mut user = match User::load() {
        Some(u) => u,
        None => User { id: 1001, active: true, level: 3 },
    };

    user.level = 4;
    user.id = 40000;
    user.store();

    // ... change local copy ...
    user.level = 5;
    user.id = 40001;

    // ... later ...

    let reloaded_user = match User::load() {
        Some(u) => u,
        None => User { id: 0, active: false, level: 0 },
    };
    require(reloaded_user.level == 4, b"user level must be 4");
    require(reloaded_user.id == 40000, b"user id must be 40000");

    // --- Config ---
    let mut config = match Config::load() {
        Some(c) => c,
        None => Config { retries: 2, timeout_ms: 3000 },
    };

    config.retries = 13;
    config.timeout_ms = 100000;
    config.store();

    // ... change local copy ...
    config.retries = 15;
    config.timeout_ms = 103000;

    // ... later ...

    let reloaded_config = match Config::load() {
        Some(c) => c,
        None => Config { retries: 2, timeout_ms: 3000 },
    };
    require(reloaded_config.retries == 13, b"config retries must be 13");
    require(reloaded_config.timeout_ms == 100000, b"config timeout_ms must be 100000");

    // // --- Session ---
    // let mut session = match Session::load() {
    //     Some(s) => s,
    //     None => Session { tx_count: 0, last_error: 0, completed: false },
    // };

    // // ... later ...

    // let reloaded_session = match Session::load() {
    //     Some(s) => s,
    //     None => Session { tx_count: 0, last_error: 0, completed: false },
    // };

    // // Compute error code
    // let error_code = (reloaded_user.level as u32)
    //                + (reloaded_config.timeout_ms as u32)
    //                + (reloaded_session.tx_count as u32);

    Result {
        success: true,
        error_code:0,//error_code,
    }
}

entrypoint!(my_vm_entry);
