//! Deterministic OS scaffold for blockchain-style execution.
//!
//! This crate provides a bootloader skeleton that:
//! - loads a kernel program into fresh pages,
//! - wires a simple syscall trap table, and
//! - hands the loaded image off to a future kernel runtime.
//!
//! Memory utilities are local copies of the VM page primitives to keep the OS
//! independent from the execution engine.

pub mod bootloader;
pub mod kernel;
pub mod memory;
pub mod traps;
