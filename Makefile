.PHONY: all

all: clean program example_programs test utils summary

clean:
	@echo "=== Cleaning project ==="
	cargo clean
	$(MAKE) -C crates/examples clean
	@cd utils/binary_comparison && $(MAKE) clean > /dev/null 2>&1 || true
	@echo "=== Clean complete ==="

program: 
	@echo "=== Building program ==="
	cargo clean -p program
	cargo build -p program --target riscv32imac-unknown-none-elf
	@echo "=== Program build complete ==="

example_programs: clean
	@echo "=== Building example programs ==="
	$(MAKE) -C crates/examples
	@echo "=== Example programs build complete ==="

test: generate_abis
	@echo "=== Running tests ==="
	cargo test -p types -p storage -p state
	cargo test -p program --lib
	cargo test -p vm --lib
	cargo test -p compiler --lib
	cd crates/examples && cargo test -- --nocapture
	@echo "=== Tests complete ==="

generate_abis:
	@echo "=== Generating ABIs ==="
	cd crates/examples && $(MAKE) abi
	@echo "=== ABI generation complete ==="

utils:
	@echo "=== Building utilities ==="
	@echo "📦 Building binary comparison tool..."
	@cd utils/binary_comparison && $(MAKE) release
	@echo "=== Utilities build complete ==="

summary:
	@echo ""
	@echo "🎉 BUILD SUMMARY"
	@echo "================"
	@echo "✅ Cleaned project artifacts"
	@echo "✅ Built program crate for RISC-V target"
	@echo "✅ Built 10 example programs:"
	@echo "   - allocator_demo: Memory allocation demonstration"
	@echo "   - calculator: Math operations contract"
	@echo "   - calculator_client: Contract that calls calculator"
	@echo "   - call_program: Cross-contract call demonstration"
	@echo "   - erc20: Token contract implementation"
	@echo "   - lib_import: External library usage (SHA256)"
	@echo "   - logging: Logging functionality test"
	@echo "   - multi_func: Multiple function routing"
	@echo "   - simple: Basic contract example"
	@echo "   - storage: Storage operations test"
	@echo "✅ Generated ABIs for all example programs"
	@echo "✅ Ran tests for all library crates:"
	@echo "   - types"
	@echo "   - storage"
	@echo "   - state"
	@echo "   - program"
	@echo "   - vm"
	@echo "   - compiler"
	@echo "✅ Ran 9 integration tests including:"
	@echo "   - ERC20 token operations"
	@echo "   - Cross-contract calls (calculator & client)"
	@echo "   - Storage operations"
	@echo "   - Memory allocation"
	@echo "   - Library imports (SHA256)"
	@echo "   - Logging demonstrations"
	@echo "✅ Built utilities:"
	@echo "   - binary_comparison: Tool for comparing VM logs with ELF binaries"
	@echo ""
	@echo "🚀 All targets completed successfully!"
	@echo ""
