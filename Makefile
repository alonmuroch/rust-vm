.PHONY: all

all: clean program example_programs test summary

program:
	@echo "=== Building program ==="
	cargo clean -p program
	cargo build -p program --target riscv32imac-unknown-none-elf
	@echo "=== Program build complete ==="

example_programs:
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

summary:
	@echo ""
	@echo "🎉 BUILD SUMMARY"
	@echo "================"
	@echo "✅ Cleaned project artifacts"
	@echo "✅ Built program crate for RISC-V target"
	@echo "✅ Built 5 example programs:"
	@echo "   - call_program"
	@echo "   - erc20"
	@echo "   - multi_func"
	@echo "   - simple"
	@echo "   - storage"
	@echo "✅ Generated ABIs for all example programs"
	@echo "✅ Ran tests for all library crates:"
	@echo "   - types"
	@echo "   - storage"
	@echo "   - state"
	@echo "   - program"
	@echo "   - vm"
	@echo "   - compiler"
	@echo "✅ Ran example tests using generated ABIs"
	@echo ""
	@echo "🚀 All targets completed successfully!"
	@echo ""

clean:
	@echo "=== Cleaning project ==="
	cargo clean
	$(MAKE) -C crates/examples clean
	@echo "=== Clean complete ==="
