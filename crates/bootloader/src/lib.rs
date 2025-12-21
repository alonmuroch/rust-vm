#![cfg_attr(target_arch = "riscv32", no_std)]
//! Deterministic OS scaffold for blockchain-style execution.
//!
//! This crate provides a bootloader skeleton that:
//! - loads a kernel program into fresh pages,
//! - hands the loaded image off to a future kernel runtime.
//!
//! Memory utilities are local copies of the VM page primitives to keep the OS
//! independent from the execution engine.

pub mod bootloader;

pub mod memory;

pub mod syscalls;

pub use vm::sys_call;

pub use syscalls::DefaultSyscallHandler;
