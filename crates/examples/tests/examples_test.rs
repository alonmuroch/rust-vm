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
                        // Test data from program/tests/ecdsa_test.rs
                        // Message: "Hello, AVM!"
                        let mut data = Vec::new();

                        // Uncompressed public key (65 bytes)
                        data.extend_from_slice(&[
                            0x04, // uncompressed prefix
                            // x coordinate (32 bytes)
                            0x46, 0x46, 0xae, 0x50, 0x47, 0x31, 0x6b, 0x42,
                            0x30, 0xd0, 0x08, 0x6c, 0x8a, 0xce, 0xc6, 0x87,
                            0xf0, 0x0b, 0x1c, 0xd9, 0xd1, 0xdc, 0x63, 0x4f,
                            0x6c, 0xb3, 0x58, 0xac, 0x0a, 0x9a, 0x8f, 0xff,
                            // y coordinate (32 bytes)
                            0xfe, 0x77, 0xb4, 0xdd, 0x0a, 0x4b, 0xfb, 0x95,
                            0x85, 0x1f, 0x3b, 0x73, 0x55, 0xc7, 0x81, 0xdd,
                            0x60, 0xf8, 0x41, 0x8f, 0xc8, 0xa6, 0x5d, 0x14,
                            0x90, 0x7a, 0xff, 0x47, 0xc9, 0x03, 0xa5, 0x59
                        ]);

                        // Valid signature (64 bytes) - r (32 bytes) + s (32 bytes)
                        data.extend_from_slice(&[
                            0xfb, 0x00, 0x99, 0xfc, 0x73, 0xf3, 0xff, 0xf0,
                            0x63, 0xe3, 0x55, 0xd4, 0x3f, 0x7b, 0xa8, 0xea,
                            0xed, 0x26, 0x8d, 0x07, 0x5d, 0xcb, 0x37, 0x87,
                            0x2c, 0xf3, 0x60, 0x66, 0x4e, 0x66, 0x17, 0x56,
                            0x08, 0xaf, 0xf6, 0xa8, 0x71, 0x85, 0x09, 0x10,
                            0xc1, 0x8c, 0xc2, 0x1c, 0x61, 0x36, 0xda, 0xad,
                            0x47, 0x0e, 0xc9, 0x24, 0x0b, 0x11, 0x79, 0xe8,
                            0xc5, 0x38, 0x79, 0xa9, 0x60, 0x80, 0xbd, 0xa2
                        ]);

                        // Message hash (32 bytes) - SHA256("Hello, AVM!")
                        data.extend_from_slice(&[
                            0xd3, 0xb6, 0xe5, 0x0f, 0x20, 0x34, 0x37, 0xd2,
                            0x5d, 0x14, 0x02, 0x4d, 0xa4, 0x90, 0x9f, 0x89,
                            0x82, 0x4f, 0x1f, 0x33, 0x73, 0xa1, 0x21, 0x37,
                            0x05, 0xd1, 0xa1, 0xd4, 0x22, 0x24, 0x85, 0x31
                        ]);

                        data
                    })(),
                    value: 0,
                    nonce: 0,
                },
            ]),
        },
    ]
});

#[test]
fn test_examples() {
    TestRunner::default().execute().unwrap()
}