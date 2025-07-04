use avm::transaction::{TransactionType, TransactionBundle, Transaction};
use avm::router::{encode_router_calls,HostFuncCall};
use types::address::Address;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct TestCase<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub expected_success: bool,
    pub expected_error_code: u32,
    pub bundle: TransactionBundle,
}

pub static TEST_CASES: Lazy<Vec<TestCase<'static>>> = Lazy::new(|| {
    vec![
        TestCase {
            name: "call program",
            path: "../examples/bin/call_program.elf",
            expected_success: true,
            expected_error_code: 0,
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: vec![],
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: vec![
                        100, 0, 0, 0,   // first u64 = 100
                        42, 0, 0, 0,      // second u64 = 42
                    ], 
                    value: 0,
                    nonce: 0,
                },
            ]),
        },

        // TestCase {
        //     name: "account create (storage)",
        //     path: "../examples/bin/storage.elf",
        //     expected_success: true,
        //     expected_error_code: 0,
        //     bundle: TransactionBundle::new(vec![
        //         Transaction {
        //             tx_type: TransactionType::CreateAccount,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: vec![],
        //             value: 0,
        //             nonce: 0,
        //         },
        //         Transaction {
        //             tx_type: TransactionType::ProgramCall,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: vec![],
        //             value: 0,
        //             nonce: 0,
        //         },
        //     ]),
        // },

        // TestCase {
        //     name: "account create (simple)",
        //     path: "../examples/bin/simple.elf",
        //     expected_success: true,
        //     expected_error_code: 100,
        //     bundle: TransactionBundle::new(vec![
        //         Transaction {
        //             tx_type: TransactionType::CreateAccount,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: vec![],
        //             value: 0,
        //             nonce: 0,
        //         },
        //         Transaction {
        //             tx_type: TransactionType::ProgramCall,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: vec![
        //                 100, 0, 0, 0,   // first u64 = 100
        //                 42, 0, 0, 0,      // second u64 = 42
        //             ], 
        //             value: 0,
        //             nonce: 0,
        //         },
        //     ]),
        // },

        // TestCase {
        //     name: "multi function (simple)",
        //     path: "../examples/bin/multi_func.elf",
        //     expected_success: true,
        //     expected_error_code: 100,
        //     bundle: TransactionBundle::new(vec![
        //         Transaction {
        //             tx_type: TransactionType::CreateAccount,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: vec![],
        //             value: 0,
        //             nonce: 0,
        //         },
        //         Transaction {
        //             tx_type: TransactionType::ProgramCall,
        //             to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
        //             data: encode_router_calls(&[
        //                 HostFuncCall {
        //                     selector: 0x01,
        //                     args: vec![
        //                         100, 0, 0, 0, // first = 100
        //                         42, 0, 0, 0,  // second = 42
        //                     ],
        //                 }
        //             ]),
        //             value: 0,
        //             nonce: 0,
        //         },
        //     ]),
        // },
    ]
});

pub fn to_address(hex: &str) -> Address {
    assert!(hex.len() == 40, "Hex string must be exactly 40 characters");

    fn from_hex_char(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("Invalid hex character"),
        }
    }

    let mut bytes = [0u8; 20];
    let hex_bytes = hex.as_bytes();
    for i in 0..20 {
        let hi = from_hex_char(hex_bytes[i * 2]);
        let lo = from_hex_char(hex_bytes[i * 2 + 1]);
        bytes[i] = (hi << 4) | lo;
    }

    Address::new(bytes)
}
