# RISC-V ISA Compliance Tests for rust-vm

This directory contains the setup and instructions for running the official [riscv-tests](https://github.com/riscv/riscv-tests) compliance suite against your RISC-V VM implementation.

## How to Build riscv-tests

1. **Ensure you have a RISC-V GCC toolchain installed** (e.g., `riscv32-unknown-elf-gcc` or `riscv64-unknown-elf-gcc`).
2. **Initialize submodules:**
   ```sh
   git submodule update --init --recursive
   ```
3. **Build and install the tests:**
   ```sh
   cd crates/vm/tests
   make full
   ```
   This will:
   - Update submodules
   - Run autoconf/configure
   - Build the tests
   - Install the ELF files to `riscv-tests-install/share/riscv-tests/isa/`

## Where Are the Test ELF Files?

After building and installing, the ELF files are located in:

```
crates/vm/tests/riscv-tests-install/share/riscv-tests/isa/
```

## How to Inspect an Individual Test ELF File

You can use `objdump` from your RISC-V toolchain to inspect the ELF files:

- **Disassemble:**
  ```sh
  riscv64-unknown-elf-objdump -d rv32ui-p-add
  ```
- **Show ELF headers:**
  ```sh
  riscv64-unknown-elf-objdump -f rv32ui-p-add
  ```

> Note: You can use `riscv32-unknown-elf-objdump` or `riscv64-unknown-elf-objdump` (with the correct flags) for RV32 ELF files.

## Types of Tests to Use

Depending on your VM's supported extensions, use the following test sets:

- `rv32ui-p-*` — Base integer instructions (RV32I)
- `rv32um-p-*` — Multiply/divide instructions (RV32IM)
- `rv32ua-p-*` — Atomic instructions (RV32IA)
- `rv32uc-p-*` — Compressed instructions (RV32IC)
- `rv32imc-p-*` — Combined tests (if present)

**Ignore `rv64*` tests** unless your VM supports 64-bit RISC-V.

## Running Tests in the VM

- Use the provided Rust test runner (e.g., `test_runner_ui.rs`) to load and execute ELF files in your VM.
- You can extend the runner to batch test all ELF files of a given type and check results automatically.

---

For more details, see the [riscv-tests README](https://github.com/riscv/riscv-tests/blob/master/README.md). 