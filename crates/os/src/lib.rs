#![cfg_attr(target_arch = "riscv32", no_std)]
//! Deterministic OS scaffold for blockchain-style execution.
//!
//! This crate provides a bootloader skeleton that:
//! - loads a kernel program into fresh pages,
//! - hands the loaded image off to a future kernel runtime.
//!
//! Memory utilities are local copies of the VM page primitives to keep the OS
//! independent from the execution engine.

#[cfg(target_arch = "riscv32")]
pub mod allocator;
#[cfg(not(target_arch = "riscv32"))]
pub mod bootloader;
#[cfg(not(target_arch = "riscv32"))]
pub mod memory;
#[cfg(not(target_arch = "riscv32"))]
pub use vm::sys_call;
#[cfg(not(target_arch = "riscv32"))]
pub mod syscalls;
#[cfg(not(target_arch = "riscv32"))]
pub use syscalls::DefaultSyscallHandler;
