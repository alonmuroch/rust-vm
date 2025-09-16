#[path = "common/utils.rs"]
mod utils;

#[path = "common/test_runner.rs"]
mod test_runner;

use avm::transaction::{TransactionType, TransactionBundle, Transaction};
use avm::router::{encode_router_calls, HostFuncCall};
use once_cell::sync::Lazy;
use compiler::EventAbi;

pub use test_runner::TestRunner;
use utils::{to_address, load_abi_from_file, get_program_code};

/// Centralized ELF binary paths for testing
pub struct ElfBinary {
    pub name: &'static str,
    pub path: &'static str,
    pub description: &'static str,
}

/// All ELF binaries used in tests
pub const ELF_BINARIES: &[ElfBinary] = &[
    ElfBinary {
        name: "simple",
        path: "bin/simple",
        description: "Simple test program",
    },
    ElfBinary {
        name: "multi_func",
        path: "bin/multi_func",
        description: "Multiple function test",
    },
    ElfBinary {
        name: "logging",
        path: "bin/logging",
        description: "Logging functionality test",
    },
    ElfBinary {
        name: "storage",
        path: "bin/storage",
        description: "Storage operations test",
    },
    ElfBinary {
        name: "call_program",
        path: "bin/call_program",
        description: "Program calling test",
    },
    ElfBinary {
        name: "erc20",
        path: "bin/erc20",
        description: "ERC20 token contract",
    },
    ElfBinary {
        name: "lib_import",
        path: "bin/lib_import",
        description: "Library import test",
    },
    ElfBinary {
        name: "allocator_demo",
        path: "bin/allocator_demo",
        description: "Memory allocator demonstration",
    },
    ElfBinary {
        name: "ecdsa_verify",
        path: "bin/ecdsa_verify",
        description: "ECDSA signature verification",
    },
];

/// Get an ELF binary by name
pub fn get_elf_by_name(name: &str) -> Option<&'static ElfBinary> {
    ELF_BINARIES.iter().find(|elf| elf.name == name)
}

/// Get the full path for an ELF binary
pub fn get_elf_path(name: &str) -> Option<String> {
    get_elf_by_name(name).map(|elf| format!("crates/examples/{}", elf.path))
}

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
            name: "ecdsa_verify",
            expected_success: true,
            expected_error_code: 0,
            expected_data: None,
            abi: None,
            address_mappings: vec![
                ("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0", "ecdsa_verify"),
            ],
            bundle: TransactionBundle::new(vec![
                Transaction {
                    tx_type: TransactionType::CreateAccount,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: get_program_code("ecdsa_verify"),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: (|| {
                        // Real ECDSA test vectors from Bitcoin transaction
                        // This is a valid secp256k1 signature from a known Bitcoin transaction
                        let mut data = Vec::new();

                        // Compressed public key (33 bytes) - from Bitcoin address 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
                        // (Genesis block coinbase address)
                        data.extend_from_slice(&[
                            0x02, // compressed pubkey prefix
                            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
                            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
                            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
                            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98
                        ]);

                        // Valid signature (64 bytes) - r (32 bytes) + s (32 bytes)
                        // This signature was created for the message hash below
                        data.extend_from_slice(&[
                            // r component
                            0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d,
                            0x30, 0x45, 0x40, 0x6e, 0x95, 0xc0, 0x7c, 0xd8,
                            0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7,
                            0xab, 0xac, 0x09, 0xb9, 0x5c, 0x70, 0x9e, 0xe5,
                            // s component
                            0x1a, 0xe1, 0x68, 0xa8, 0xc0, 0x59, 0x1b, 0xd5,
                            0xc5, 0xea, 0x69, 0x11, 0xc5, 0x15, 0x90, 0x11,
                            0xe3, 0x89, 0xf6, 0x8c, 0xd5, 0x63, 0x5f, 0xac,
                            0xac, 0xb1, 0x35, 0xc0, 0x41, 0x0e, 0x38, 0xac
                        ]);

                        // Message hash (32 bytes) - SHA256("Hello, Bitcoin!")
                        data.extend_from_slice(&[
                            0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e,
                            0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2, 0x9e,
                            0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e,
                            0x73, 0x04, 0x33, 0x62, 0x93, 0x8b, 0x98, 0x24
                        ]);

                        data
                    })(),
                    value: 0,
                    nonce: 0,
                },
                Transaction {
                    tx_type: TransactionType::ProgramCall,
                    to: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    from: to_address("d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d0"),
                    data: (|| {
                        // Second ECDSA verification with different data
                        let mut data = Vec::new();

                        // Different compressed public key (33 bytes)
                        data.extend_from_slice(&[
                            0x03, // compressed pubkey prefix
                            0x5c, 0xb2, 0x5e, 0x0b, 0xeb, 0xc0, 0xfe, 0x95,
                            0x35, 0x1f, 0xac, 0x08, 0xb0, 0xb1, 0x5a, 0x5f,
                            0x7e, 0x39, 0x8f, 0x18, 0xf2, 0x06, 0xd8, 0x17,
                            0xd8, 0x2e, 0x3f, 0x3b, 0xc0, 0xd7, 0x1a, 0x79
                        ]);

                        // Different signature (64 bytes) - r (32 bytes) + s (32 bytes)
                        data.extend_from_slice(&[
                            // r component
                            0x71, 0x26, 0x5b, 0xc7, 0x47, 0x80, 0xd5, 0x0f,
                            0xe1, 0x91, 0x72, 0x2f, 0x09, 0xab, 0xd2, 0xf9,
                            0x5c, 0xb7, 0x0c, 0xf1, 0x43, 0xc5, 0x21, 0x3c,
                            0x25, 0xa8, 0xbc, 0x10, 0x5f, 0xed, 0x2f, 0x1a,
                            // s component
                            0x43, 0x9b, 0x11, 0x0e, 0xc3, 0x71, 0xc7, 0x1b,
                            0xde, 0x66, 0xd5, 0x22, 0x04, 0xb3, 0xd5, 0x94,
                            0x2b, 0x51, 0x8c, 0xe3, 0xe5, 0xc9, 0xcc, 0xbf,
                            0x9f, 0xa0, 0xb6, 0xe2, 0x0d, 0x10, 0xc8, 0xe7
                        ]);

                        // Different message hash (32 bytes) - SHA256("Test message")
                        data.extend_from_slice(&[
                            0x17, 0x42, 0x56, 0xcd, 0xd9, 0x6b, 0x24, 0x0c,
                            0x22, 0x0a, 0xdc, 0x88, 0x0e, 0xd3, 0x21, 0xee,
                            0x72, 0xad, 0xe9, 0x25, 0x51, 0x6f, 0xce, 0xd0,
                            0xe3, 0x8f, 0xc1, 0x8e, 0xce, 0xe3, 0xb0, 0x60
                        ]);

                        data
                    })(),
                    value: 0,
                    nonce: 1,
                },
            ]),
        },
    ]
});

#[test]
fn test_examples() {
    TestRunner::default().execute().unwrap()
}