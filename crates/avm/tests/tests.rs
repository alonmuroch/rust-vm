use avm::transaction::{TransactionType, TransactionBundle, Transaction};
use avm::router::{encode_router_calls,HostFuncCall};
use types::address::Address;
use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::global::Config;
use compiler::{EventParam, EventAbi, ParamType};

#[derive(Debug)]
pub struct TestCase<'a> {
    pub name: &'a str,
    pub expected_success: bool,
    pub expected_error_code: u32,
    pub bundle: TransactionBundle,
    pub abi: Option<Vec<EventAbi>>
}

pub static TEST_CASES: Lazy<Vec<TestCase<'static>>> = Lazy::new(|| {
    vec![
        TestCase {
            name: "erc20",
            expected_success: true,
            expected_error_code: 0,
            abi: Some(vec!(
                EventAbi {
                    name: "Minted".to_string(),
                    inputs: vec![
                        EventParam {
                            name: "caller".to_string(),
                            kind: ParamType::Address,
                            indexed: false,
                        },
                        EventParam {
                            name: "amount".to_string(),
                            kind: ParamType::Uint(32),
                            indexed: false,
                        },
                    ],
                },
                EventAbi {
                    name: "Transfer".to_string(),
                    inputs: vec![
                        EventParam {
                            name: "from".to_string(),
                            kind: ParamType::Address,
                            indexed: false,
                        },
                        EventParam {
                            name: "to".to_string(),
                            kind: ParamType::Address,
                            indexed: false,
                        },
                        EventParam {
                            name: "value".to_string(),
                            kind: ParamType::Uint(32),
                            indexed: false,
                        },
                    ],
                }
            )),
            bundle: TransactionBundle::new(vec![
                 Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/erc20"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x01,
                            args: (|| {
                                // max supply 
                                let max_supply: u32 = 100000000; // 100 million
                                let mut max_supply_bytes: Vec<u8> = max_supply.to_le_bytes().to_vec();

                                // decimals
                                let decimals: u8 = 18;

                                // combine
                                max_supply_bytes.extend(vec![decimals]);
                                max_supply_bytes
                            })(),
                        }
                    ]),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x02, // transfer
                            args: (|| {
                                // to address (20 bytes)
                                let to_addr = to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d2");
                                let mut args = to_addr.0.to_vec();
                                
                                // amount (4 bytes)
                                let amount: u32 = 50000000; // 50 million tokens
                                args.extend(amount.to_le_bytes());
                                
                                args
                            })(),
                        }
                    ]),
                    value: 0,
                    nonce: 0,
                },
            ]),
        },

        TestCase {
            name: "call program",
            expected_success: true,
            expected_error_code: 100,
            abi: None,
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/call_program"),
                    value: 0,
                    nonce: 0,
                },
                 Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/simple"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: (|| {
                        let mut data = to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1").0.to_vec();
                        data.extend(vec![100, 0, 0, 0, 42, 0, 0, 0]);
                        data
                    })(),
                    value: 0,
                    nonce: 0,
                },
            ]),
        },

        TestCase {
            name: "account create (storage)",
            expected_success: true,
            expected_error_code: 0,
            abi: None,
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/storage"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: vec![],
                    value: 0,
                    nonce: 0,
                },
            ]),
        },

        TestCase {
            name: "account create (simple)",
            expected_success: true,
            expected_error_code: 100,
            abi: None,
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/simple"),
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

        TestCase {
            name: "multi function (simple)",
            expected_success: true,
            expected_error_code: 100,
            abi: None,
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("../../target/riscv32imac-unknown-none-elf/release/multi_func"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x01,
                            args: vec![
                                100, 0, 0, 0, // first = 100
                                42, 0, 0, 0,  // second = 42
                            ],
                        }
                    ]),
                    value: 0,
                    nonce: 0,
                },
            ]),
        },
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

    Address(bytes)
}

pub fn get_program_code<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let path_str = path.as_ref().display();
    let bytes = fs::read(&path)
        .unwrap_or_else(|_| panic!("❌ Failed to read ELF file from {}", path_str));

    let elf = parse_elf_from_bytes(&bytes)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ELF from {}", path_str));

    let (code, code_start) = elf
    .get_flat_code()
    .unwrap_or_else(|| panic!("❌ No code sections found in ELF {}", path_str));

    let (rodata, rodata_start) = elf
        .get_flat_rodata()
        .unwrap_or_else(|| {
            (vec![], usize::MAX as u64)
        });

    // assert sizes
    assert!(code.len() <= Config::CODE_SIZE_LIMIT, "code size exceeds limit");
    assert!(rodata.len() <= Config::RO_DATA_SIZE_LIMIT, "read only data size exceeds limit");

    let mut total_len = code_start + code.len() as u64; // assumes rodata is after code
    if rodata.len() > 0 {
        total_len = rodata_start + rodata.len() as u64; // assumes rodata is after code
    }

    // Initialize memory with 0x00
    let mut combined = vec![0u8; total_len as usize];

    // Copy code
    combined[code_start as usize..code_start as usize + code.len()].copy_from_slice(&code);

    // Copy rodata (if it exists)
    if rodata.len() > 0 {
        combined[rodata_start as usize..rodata_start as usize + rodata.len()].copy_from_slice(&rodata);
    }
    combined
}