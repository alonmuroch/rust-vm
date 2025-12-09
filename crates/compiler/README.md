# Compiler crate

This crate holds the tooling used to compile AVM contracts, generate ABIs, and emit Rust client code for interacting with deployed programs.

## ABI generation

The ABI generator (`abi_generator.rs`) is a lightweight source analyzer that walks a contract's Rust source to find routed functions and events:
- It scans for `event!` macro invocations and records each event name and field types.
- It looks for the `route(...)` call pattern inside `main_entry` and matches selector arms (e.g., `0x01 => { ... }`), then discovers the callee functions invoked inside each arm.
- For each routed function, it parses the function signature to collect inputs and outputs. The implicit `caller: Address` argument is omitted from the ABI so generated clients only encode the routed arguments.
- It preserves return types when they map to known ABI types (e.g., `Result` or `u32`), so consumers know how to decode responses.

Run it directly via the `abi` subcommand of the `avm32` binary:
```
cargo run -p compiler --bin avm32 -- abi --bin erc20 --manifest-path <path/to/Cargo.toml> --src <path/to/erc20.rs> --out <path/to/output/erc20.abi.json>
```
If `--src` is omitted, it infers `<manifest_dir>/src/<bin>.rs`. The output is a JSON ABI with functions and events.

## Generated ABI client code

`abi_codegen.rs` consumes an ABI JSON and emits a small Rust client with helper methods for each routed function. The `client` subcommand wires this up:
```
cargo run -p compiler --bin avm32 -- client --abi <path/to/erc20.abi.json> --out <path/to/erc20_abi.rs> --contract Erc20Contract
```
In the examples, the DEX (`crates/examples/src/dex.rs`) includes the generated `erc20_abi.rs` client and calls `Erc20Contract::transfer`/`balance_of` to interact with the ERC20 program. You can follow that pattern to integrate the generated client into your own code.

## avm32 compiler CLI

`avm32` is a small convenience wrapper around Cargo and the ABI/codegen tools. It defaults to using the manifest in the current working directory (falls back to the workspace root) and outputs to `<manifest_dir>/bin`. All commands accept `--manifest-path` to override, and `--linker-script` to point at a custom script (defaults to `crates/compiler/linker.ld`).

Subcommands:
- `build`: cross-compiles a contract to the AVM target.
  ```
  cargo run -p compiler --bin avm32 -- build --bin erc20 --manifest-path <path/to/Cargo.toml> --linker-script crates/compiler/linker.ld --out-dir <manifest_dir>/bin
  ```
- `abi`: parses source to emit `<bin>.abi.json`.
- `client`: turns an ABI JSON into a Rust client.
- `all`: runs build → abi → client in one step.

All build commands target `crates/compiler/targets/avm32.json` and pass `-Zbuild-std` flags so the core/alloc toolchain is bundled. The linker script provides the layout expected by the VM; if you customize memory layout, pass your script via `--linker-script`.
