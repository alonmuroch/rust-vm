#![no_std]
#![no_main]

extern crate program;
use program::{entrypoint, event, 
    fire_event, log, logf, persist_struct, 
    require, router::route, DataParser,
    types::{address::Address, o::O, result::Result}, vm_panic, Map};


// Persistent structs
persist_struct!(Metadata {
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
            Result::new(true, 0)
        },
        0x02 => {
            let mut parser = DataParser::new(call.args);
            let to = parser.read_address();
            let amount = parser.read_u32();
            transfer(caller, to, amount);
            Result::new(true, 0)
        },
        0x05 => {
            let mut parser = DataParser::new(call.args);
            let owner = parser.read_address();
            let b = balance_of(owner);
            Result::with_u32(b)
        },
        _ => vm_panic(b"unknown selector"),
    })
}

fn init(caller: Address, args: &[u8]) {
    logf!("init called");
    let mut meta = match Metadata::load() {
        O::Some(m) => vm_panic(b"already initialized"),
        O::None => Metadata { total_supply: 0, decimals: 0 },
    };
    
    logf!("initializing");

    let mut parser = DataParser::new(args);
    let total_supply = parser.read_u32();
    let decimals = parser.read_bytes(1)[0];

    logf!("total supply: %d", total_supply);
    logf!("decimals: %d", decimals);

    meta.total_supply = total_supply;
    meta.decimals = decimals;
    meta.store();

    // mint to caller
    mint(caller, total_supply);
}

fn mint(caller: Address, val: u32) {
    logf!("minting: %d tokens", val);
    fire_event!(Minted::new(caller, val));
    Balances::set(caller, val);
}

fn transfer(caller: Address, to: Address, amount: u32) {
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

fn balance_of(owner: Address) -> u32 {
    match Balances::get(owner) {
        O::Some(bal) => bal,
        O::None => 0,
    }
}
// ---- Entry point ----
entrypoint!(main_entry);
