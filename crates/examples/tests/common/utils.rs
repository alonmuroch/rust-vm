use types::address::Address;
use std::fs;
use std::path::Path;
use compiler::elf::parse_elf_from_bytes;
use avm::global::Config;
use compiler::{EventParam, EventAbi, ParamType};
use serde_json::Value;

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