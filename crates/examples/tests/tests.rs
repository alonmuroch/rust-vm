use avm::transaction::{TransactionType, TransactionBundle, Transaction};
use avm::router::{encode_router_calls,HostFuncCall};
use types::address::Address;
use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::global::Config;
use compiler::{EventParam, EventAbi, ParamType};
use serde_json::Value;

#[derive(Debug)]
pub struct TestCase<'a> {
    pub name: &'a str,
    pub expected_success: bool,
    pub expected_error_code: u32,
    pub expected_data: Option<Vec<u8>>,
    pub bundle: TransactionBundle,
    pub abi: Option<Vec<EventAbi>>,
    pub address_mappings: Vec<(&'a str, &'a str)>, // (address, binary_name)
}

pub static TEST_CASES: Lazy<Vec<TestCase<'static>>> = Lazy::new(|| {
    vec![
        TestCase {
            name: "erc20",
            expected_success: true,
            expected_error_code: 0,
            expected_data: Some(vec![128, 240, 250, 2]), // Expected data: 50,000,000 in little-endian
            abi: load_abi_from_file("bin/erc20.abi.json"),
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1", "erc20"),
            ],
            bundle: TransactionBundle::new(vec![
                 Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    data: get_program_code("erc20"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x01, // initialize
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
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x05, // balance_of
                            args: (|| {
                                // check balance of the original caller (d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0)
                                let owner_addr = to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0");
                                owner_addr.0.to_vec()
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
            expected_error_code: 0,
            expected_data: Some(vec![100, 0, 0, 0]), // Expected data: 100 in little-endian
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "call_program"),
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1", "simple"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("call_program"),
                    value: 0,
                    nonce: 0,
                },
                 Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d1"),
                    data: get_program_code("simple"),
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
            expected_data: None,
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "storage"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("storage"),
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
            expected_error_code: 0,
            expected_data: Some(vec![100, 0, 0, 0]), // Expected data: 100 in little-endian
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "simple"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("simple"),
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
            expected_error_code: 0,
            expected_data: Some(vec![100, 0, 0, 0]), // Expected data: 100 in little-endian
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "multi_func"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("multi_func"),
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

        TestCase {
            name: "allocator demo",
            expected_success: true,
            expected_error_code: 0,
            expected_data: None,//Some(b"VM allocator demo completed successfully!".to_vec()),
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "allocator_demo"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("allocator_demo"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: vec![], // No input data needed
                    value: 0,
                    nonce: 0,
                },
            ]),
        },

        TestCase {
            name: "lib_import - SHA256 of 'hello world'",
            expected_success: true,
            expected_error_code: 0,
            // SHA-256 hash of "hello world" = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
            expected_data: Some(vec![
                0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
                0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
                0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
                0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9
            ]),
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "lib_import"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("lib_import"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: b"hello world".to_vec(), // Input data to hash
                    value: 0,
                    nonce: 0,
                },
            ]),
        },
        TestCase {
            name: "logging demo",
            expected_success: true,
            expected_error_code: 0,
            expected_data: None, // Logging doesn't return specific data
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0e0", "logging"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0e0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("logging"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0e0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF], // Some test data
                    value: 0,
                    nonce: 0,
                },
            ]),
        },
        TestCase {
            name: "calculator with client",
            expected_success: true,
            expected_error_code: 0,
            expected_data: Some(vec![15, 0, 0, 0]), // Expected: 10 + 5 = 15
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c1", "calculator"),
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c2", "calculator_client"),
            ],
            bundle: TransactionBundle::new(vec![
                // Deploy calculator contract
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c1"), // Calculator address
                    data: get_program_code("calculator"),
                    value: 0,
                    nonce: 0,
                },
                // Deploy calculator client contract
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c2"), // Client address
                    data: get_program_code("calculator_client"),
                    value: 0,
                    nonce: 0,
                },
                // Call calculator directly to test it works (5 + 3 = 8)
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c1"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: encode_router_calls(&[
                        HostFuncCall {
                            selector: 0x01, // add
                            args: {
                                let mut args = Vec::new();
                                args.extend_from_slice(&5u32.to_le_bytes());
                                args.extend_from_slice(&3u32.to_le_bytes());
                                args
                            },
                        }
                    ]),
                    value: 0,
                    nonce: 0,
                },
                // Call calculator client to call calculator (10 + 5 = 15)
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c2"), // Client
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: {
                        let mut data = Vec::new();
                        // Calculator contract address (20 bytes)
                        data.extend_from_slice(&to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0c1").0);
                        // Operation: 1 = add
                        data.push(1);
                        // First operand: 10
                        data.extend_from_slice(&10u32.to_le_bytes());
                        // Second operand: 5
                        data.extend_from_slice(&5u32.to_le_bytes());
                        data
                    },
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

pub fn load_abi_from_file<P: AsRef<Path>>(path: P) -> Option<Vec<EventAbi>> {
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("❌ Failed to read ABI file from {}", path.as_ref().display()));
    
    let json: Value = serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ABI JSON from {}", path.as_ref().display()));
    
    let events = json.get("events")?;
    let events_array = events.as_array()?;
    
    let mut event_abis = Vec::new();
    for event in events_array {
        let name = event.get("name")?.as_str()?.to_string();
        let inputs = event.get("inputs")?.as_array()?;
        
        let mut params = Vec::new();
        for input in inputs {
            let param_name = input.get("name")?.as_str()?.to_string();
            let param_type_str = input.get("type")?.as_str()?;
            let indexed = input.get("indexed").and_then(|v| v.as_bool()).unwrap_or(false);
            
            let param_type = match param_type_str {
                "address" => ParamType::Address,
                "uint32" => ParamType::Uint(32),
                "uint64" => ParamType::Uint(64),
                "uint128" => ParamType::Uint(128),
                "uint256" => ParamType::Uint(256),
                "bool" => ParamType::Bool,
                "string" => ParamType::String,
                "bytes" => ParamType::Bytes,
                _ => panic!("❌ Unsupported parameter type: {}", param_type_str),
            };
            
            params.push(EventParam {
                name: param_name,
                kind: param_type,
                indexed,
            });
        }
        
        event_abis.push(EventAbi {
            name,
            inputs: params,
        });
    }
    
    Some(event_abis)
}

pub fn get_program_code(name: &str) -> Vec<u8> {
    // Build the full path
    let bin_path = format!("bin/{}", name);
    
    // Try reading from bin directory first (for compiled binaries)
    let bytes = fs::read(&bin_path)
        .or_else(|_| {
            // Fallback to target directory for development
            let target_path = format!("../../target/riscv32imac-unknown-none-elf/release/{}", name);
            fs::read(&target_path)
        })
        .unwrap_or_else(|_| panic!("❌ Failed to read ELF file: {}", name));

    let elf = parse_elf_from_bytes(&bytes)
        .unwrap_or_else(|_| panic!("❌ Failed to parse ELF from {}", name));

    let (code, code_start) = elf
    .get_flat_code()
    .unwrap_or_else(|| panic!("❌ No code sections found in ELF {}", name));

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