#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, event, 
    fire_event, log, logf, persist_struct, 
    read_u32, require, router::route, 
    types::{address::Address, o::O, result::Result}, vm_panic, Map};


// Persistent structs
static PERSIST_METADATA: [u8; 9] = *b"metadata\0";
persist_struct!(Metadata, PERSIST_METADATA, {
    total_supply: u32,
    decimals: u8,
});

event!(Minted {
    caller => Address,
    amount => u32,
});

event!(Transfer {
    from => Address,
    to => Address,
    value => u32,
});

Map!(Balances);

unsafe fn main_entry(program: Address, caller: Address, data: &[u8]) -> Result {   
    route(data, program, caller, 
         |to, from, call| match call.selector {
        0x01 => {
            init(caller, call.args);
            Result { success: true, error_code: 0 }
        },
        0x02 => {
            transfer(caller, call.args);
            Result { success: true, error_code: 0 }
        },
        0x05 => {
            let b = balance_of(call.args);
            Result { success: true, error_code: b as u32 }
        },
        _ => vm_panic(b"unknown selector"),
    });
    Result { success: true, error_code: 0 }
}

fn init(caller: Address, args: &[u8]) {
    logf!(b"init called");
    let mut meta = match Metadata::load() {
        O::Some(m) => vm_panic(b"already initialized"),
        O::None => Metadata { total_supply: 0, decimals: 0 },
    };
    
    logf!(b"initializing");

    let total_supply = read_u32(&args[0..4]);
    let decimals = args[4];

    logf!(b"total supply: %d", total_supply);
    logf!(b"decimals: %d", decimals);

    meta.total_supply = total_supply;
    meta.decimals = decimals;
    meta.store();

    // mint to caller
    mint(caller, total_supply);
}

fn mint(caller: Address, val: u32) {
    logf!(b"minting: %d tokens", val);
    fire_event!(Minted::new(caller, val));
    Balances::set(caller, val);
}

fn transfer(caller: Address, args: &[u8]) {
    let to = Address::from_ptr(&args[..20]).expect("Invalid address format");
    let amount = read_u32(&args[20..24]);

    let from_bal = match Balances::get(caller) {
        O::Some(bal) => bal,
        O::None => 0,
    };
    
    if from_bal < amount {
        vm_panic(b"insufficient");
    }

    let to_bal = match Balances::get(to) {
        O::Some(bal) => bal,
        O::None => 0,
    };
    
    Balances::set(caller, from_bal - amount);
    Balances::set(to, to_bal + amount);
    
    fire_event!(Transfer::new(caller, to, amount));
}

fn balance_of(args: &[u8]) -> u32 {
    let owner = Address::from_ptr(&args[..20]).expect("Invalid address format");
    match Balances::get(owner) {
        O::Some(bal) => bal,
        O::None => 0,
    }
}
// ---- Entry point ----
entrypoint!(main_entry);