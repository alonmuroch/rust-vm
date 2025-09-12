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
        name: "calculator",
        path: "bin/calculator",
        description: "Calculator contract",
    },
    ElfBinary {
        name: "calculator_client",
        path: "bin/calculator_client",
        description: "Calculator client contract",
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
];

/// Get an ELF binary by name
pub fn get_elf_by_name(name: &str) -> Option<&'static ElfBinary> {
    ELF_BINARIES.iter().find(|elf| elf.name == name)
}

/// Get the full path for an ELF binary
pub fn get_elf_path(name: &str) -> Option<String> {
    get_elf_by_name(name).map(|elf| format!("crates/examples/{}", elf.path))
}