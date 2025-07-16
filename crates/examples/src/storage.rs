#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, types::result::Result, require};
use program::types::address::Address; 
use program::persist_struct;

pub static PERSIST_USER: [u8; 5] = *b"user\0";
pub static PERSIST_CONFIG: [u8; 7] = *b"config\0";
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

fn my_vm_entry(_self_address: Address, _caller: Address, _data: &[u8]) -> Result {
    // --- User ---
    require(User::load().is_none() == true, b"user already exists");
    let mut user = User{id: 1000, active: false, level: 3};
    user.level = 4;
    user.id = 40000;
    user.store();

    // ... change local copy ...
    user.level = 5;
    user.id = 40001;

    // ... later ...

    let reloaded_user =  User::load().expect("user not found");
    require(reloaded_user.level == 4, b"user level must be 4");
    require(reloaded_user.id == 40000, b"user id must be 40000");

    // --- Config ---
    require(Config::load().is_none() == true, b"config already exists");
    let mut config =  Config{retries: 10, timeout_ms: 10};
    config.retries = 13;
    config.timeout_ms = 100000;
    config.store();

    // ... change local copy ...
    config.retries = 15;
    config.timeout_ms = 103000;

    // ... later ...

    let reloaded_config =  Config::load().expect("config not found");
    require(reloaded_config.retries == 13, b"config retries must be 13");
    require(reloaded_config.timeout_ms == 100000, b"config timeout_ms must be 100000");

    Result {
        success: true,
        error_code:0,
    }
}

entrypoint!(my_vm_entry);
