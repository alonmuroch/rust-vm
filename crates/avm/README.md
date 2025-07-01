# AVM - Alon's Virtual Machine

AVM is a custom virtual machine designed to execute Rust-compiled contracts in a sandboxed, stack-based environment. It features an isolated execution model where each contract call is executed within its own independent VM context.

---

## 🧠 Core Concepts

### Execution Stack

AVM maintains an **execution stack** that tracks active `ExecutionContext` frames. Each time a contract calls another contract, a **new context** is pushed onto the stack.

- The **top of the stack** is the currently executing context.
- When a call finishes, the context is popped and control returns to the caller.
- This model ensures **synchronous execution** and supports **reentrancy** and **deep contract composition**.

### ExecutionContext

An `ExecutionContext` is a complete, self-contained VM instance with its own:

- **Registers** (`[u64; 32]`)
- **Program Counter (`PC`)**
- **Memory Page** (linear memory, typically fixed-size per context)
- **Stack Pointer / Heap management**
- **Gas metering** (optional but recommended)
- **Access to the syscall interface**

Each context behaves like a separate "machine" running in isolation.

---

## 🔄 Contract-to-Contract Calls

When Contract A calls Contract B:
1. A new `ExecutionContext` is initialized with:
   - B's bytecode loaded
   - Fresh memory and registers
   - Arguments passed via memory or registers
2. The new context is **pushed** onto the execution stack.
3. Execution begins in the new context.
4. Upon return, the result is passed back to the caller, and the callee context is **popped**.

This mechanism provides:
- Full isolation between contracts
- Clean memory separation (no shared heap or stack)
- Easy error handling (unwinding stack on `panic`)

---

## 📦 Memory Model

Each context is allocated its own **linear memory page**, which includes:

- `.text` (program code, optional if interpreted)
- `.rodata` and statics (copied at load time)
- Stack (grows down from high memory)
- Heap (grows up from a defined base)

All memory access is **local to the context**, preventing accidental overwrites between contracts.

---

## 🚀 Features (Planned or In Progress)

- [x] RISC-V instruction decoding (32-bit and compressed)
- [x] Memory-mapped syscall interface
- [x] Per-context execution model
- [ ] Gas accounting and metering
- [ ] Persistent storage via key-based syscalls
- [ ] Support for `vm_panic` and return codes
- [ ] Debug output and tracing

---

## 🧪 Example Use Case

```rust
#[contract]
fn contract_a() {
    call_contract("contract_b", &[arg1, arg2]);
}
