---
description: Rules and conventions for RISC-V VM development in Rust
globs: src/vm/**/*.rs
alwaysApply: false
---

# RISC-V VM Development Rules

These rules apply to the `riscv-vm` crate implementing a blockchain smart contract VM.

---

## 🧠 General Principles
- Design for **determinism**: avoid sources of non-determinism like wall-clock time, random number generators, or OS-specific syscalls
- All computation must be **verifiable** and **sandboxed**
- Do not use `unsafe` unless explicitly reviewed and documented
- Prefer pure functions for opcode execution (side-effect free where possible)

---

## 🏗️ Instruction Decoding
- Use enums to model RISC-V opcodes (e.g. `enum Instruction { ADD, SUB, ... }`)
- Use pattern matching for dispatch: `match instr { Instruction::ADD => ... }`
- Invalid opcodes must return a `Trap::IllegalInstruction` or similar error
- Decode must not panic; return `Result<Instruction, DecodeError>`

---

## 🧮 Execution Semantics
- Each opcode handler must:
  - Take and return explicit `VMState` (immutable-in, mutable-out)
  - Enforce overflow and alignment checks per RISC-V spec
  - Write unit tests for each instruction
- Use **gas metering** inside opcode execution:
  - Include a `gas_cost()` function per instruction
  - Reject execution when gas budget is exceeded
- Access to memory or registers must be bounds-checked

---

## 📦 Memory Model
- VM memory must:
  - Be page-based or flat, but explicitly bounded
  - Panic-free: all access must return a `Result`
  - Enforce aligned loads/stores (e.g., `lw` requires 4-byte alignment)
- Smart contract memory should not be allowed to grow unbounded
- Memory writes must be auditable (e.g., log writes for potential tracing)

---

## 💾 I/O and Syscalls
- All host interactions (storage, precompiles, logs) must go through an `ExternalInterface` trait
- Disallow I/O calls inside opcode handlers directly
- Only allow whitelisted syscalls (e.g. `get_storage`, `emit_event`, `call_contract`)
- Trap on unknown or unsupported syscall IDs

---

## 🔐 Security
- Prohibit:
  - Any host FFI or `std::fs`, `std::net`, `std::process`
  - Allocation outside the pre-allocated memory model
  - Use of `Rc`, `RefCell` or global mutable state inside VM core
- Validate contract binaries before execution (e.g., memory bounds, max code size)

---

## 🧪 Testing
- Each instruction must be covered by:
  - Unit test with valid inputs
  - Unit test with invalid inputs (e.g., overflow, alignment errors)
- Add integration tests:
  - End-to-end programs executed inside VM
  - Host-guest syscall interaction
- VM execution must be reproducible with same inputs

---

## 📝 Logging & Debugging
- Avoid `println!`; use structured logging via `tracing::event!` or similar
- Optionally support trace-mode for step-by-step instruction execution
- Debugging outputs must be toggleable and not impact determinism

---

## ⛽ Gas Accounting
- Include gas cost model in opcode definitions (`Instruction::gas_cost(&self) -> u64`)
- Ensure that each opcode handler decrements gas and traps on exhaustion
- Prevent gas underflow (use checked subtraction)

---

## 📚 Documentation
- Each opcode must have a doc comment explaining semantics and gas cost
- Public interfaces (`VM`, `ExternalInterface`, `Instruction`) must be documented
- Use `///` and `//!` Rustdoc conventions
