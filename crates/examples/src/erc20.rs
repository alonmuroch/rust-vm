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

Map!(Balances);

unsafe fn main_entry(program: Address, caller: Address, data: &[u8]) -> Result {   
    route(data, program, caller, 
         |to, from, call| match call.selector {
        0x01 => {
            init(caller, call.args);
            Result { success: true, error_code: 0 }
        },
        // 0x02 => {
        //     transfer(caller, call.args);
        //     Result { success: true, error_code: 0 }
        // },
        // 0x05 => {
        //     let b = balance_of(call.args);
        //     Result { success: true, error_code: b as u32 }
        // },
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

// fn transfer(caller: Address, args: &[u8]) {
//     let to = Address::from_ptr(&args[..20]).expect("Invalid address format");
//     let amount = read_u32(&args[20..28]);

//     let mut balances = Balances::load().expect("balances not found");
//     let from_bal = get_balance(&balances, caller);
//     if from_bal < amount {
//         vm_panic(b"insufficient");
//     }

//     insert_balance(&mut balances, caller, from_bal - amount);
//     let to_bal = get_balance(&balances, to);
//     insert_balance(&mut balances, to, to_bal + amount);

//     balances.store();
// }

// fn balance_of(args: &[u8]) -> u32 {
//     let owner = Address::from_ptr(&args[..20]).expect("Invalid address format");
//     let balances = Balances::load().expect("balances not found");
//     get_balance(&balances, owner)
// }

// // ---- Internal helpers ----

// fn get_balance(bal: &Balances, addr: Address) -> u32 {
//     for (a, v) in bal.entries.iter() {
//         if *a == addr {
//             return *v;
//         }
//     }
//     0
// }

// fn insert_balance(bal: &mut Balances, addr: Address, val: u32) {
//     for (a, v) in bal.entries.iter_mut() {
//         if *a == addr {
//             *v = val;
//             return;
//         }
//     }
//     for slot in bal.entries.iter_mut() {
//         if slot.0 == Address([0u8; 20]) {
//             *slot = (addr, val);
//             return;
//         }
//     }
//     vm_panic(b"no space");
// }
// ---- Entry point ----
entrypoint!(main_entry);