#![forbid(unsafe_code)]
//! Alon's OS (aOS): a deterministic, blockchain-first operating system.
//!
//! This crate hosts the top-level types and module layout for aOS. It will grow to
//! replace the current `avm` orchestrator while internalizing the `program` crate as
//! the `liba` standard library for applications.

/// Marker type for the OS surface. It will eventually own the boot/kernel/runtime
/// wiring that `avm` performs today.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Aos;

/// Bootloader-focused types used to validate and hand off control to the kernel.
pub mod bootloader {
    /// Build-time configuration the bootloader uses while measuring the kernel and
    /// standard library images.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BootConfig {
        /// Whether the bootloader should emit early console logs.
        pub debug_console: bool,
        /// Whether the bootloader may proceed with unsigned developer images.
        pub allow_developer_images: bool,
    }

    impl Default for BootConfig {
        fn default() -> Self {
            Self {
                debug_console: true,
                allow_developer_images: false,
            }
        }
    }

    /// Minimal manifest created by the bootloader and consumed by the kernel.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BootInfo<'a> {
        /// Verified kernel image payload.
        pub kernel_image: &'a [u8],
        /// Verified `liba` (standard library) image payload.
        pub liba_image: &'a [u8],
        /// Boot-time options that tailor kernel behavior.
        pub config: BootConfig,
    }

    impl<'a> BootInfo<'a> {
        /// Creates a new manifest describing the verified runtime artifacts.
        pub fn new(kernel_image: &'a [u8], liba_image: &'a [u8], config: BootConfig) -> Self {
            Self {
                kernel_image,
                liba_image,
                config,
            }
        }
    }
}

/// Kernel-facing structures for deterministic, capability-scoped execution.
pub mod kernel {
    /// Configuration that controls which services the kernel exposes to runtimes.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct KernelConfig {
        /// Whether to emit receipts and event logs.
        pub receipts_enabled: bool,
        /// Whether to allow cross-program calls.
        pub cross_program_calls_enabled: bool,
        /// Whether to allow host crypto/syscall helpers.
        pub host_functions_enabled: bool,
    }

    impl Default for KernelConfig {
        fn default() -> Self {
            Self {
                receipts_enabled: true,
                cross_program_calls_enabled: true,
                host_functions_enabled: true,
            }
        }
    }

    /// Placeholder for the kernel instance that will own scheduling and capability setup.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Kernel;
}

/// Runtime-level types that coordinate block execution and VM orchestration.
pub mod runtime {
    /// Execution context provided to each transaction/program.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ExecutionContext {
        /// Height or slot of the block being executed.
        pub height: u64,
        /// Hash of the parent state root for reproducibility.
        pub parent_state_root: [u8; 32],
        /// Index of the transaction inside the block.
        pub tx_index: u32,
    }

    impl ExecutionContext {
        /// Builds a new execution context for a transaction.
        pub fn new(height: u64, parent_state_root: [u8; 32], tx_index: u32) -> Self {
            Self {
                height,
                parent_state_root,
                tx_index,
            }
        }
    }

    /// Placeholder for the runtime that will supersede the `avm` orchestrator.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Runtime;
}

/// `liba` is the application-facing standard library derived from the existing
/// `program` crate.
pub mod liba {
    /// Enumeration of syscalls that applications may perform through `liba`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Syscall {
        /// Persist or retrieve data from the underlying storage.
        Storage,
        /// Emit events/logs that become part of transaction receipts.
        Log,
        /// Perform cryptographic helper operations.
        Crypto,
        /// Invoke another program within the same block execution.
        CrossProgramCall,
    }

    /// Minimal handle for the standard library surface exposed to applications.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Liba;
}
