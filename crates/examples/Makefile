BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

# Build all example programs for RISC-V
all: clean
	@echo "📝 Compiling these source files:"
	@echo " - call_program.rs"
	@echo " - erc20.rs"
	@echo " - multi_func.rs"
	@echo " - simple.rs"
	@echo " - storage.rs"
	@echo "🎯 Compiling src/call_program.rs to ELF..."
	RUSTFLAGS="-C link-arg=-T/Users/alonmuroch/Projects/rust-vm/crates/examples/linker.ld" \
		cargo build --release \
			--target riscv32imac-unknown-none-elf \
			--manifest-path Cargo.toml \
			--bin call_program \
			--features binaries
	@echo "🎯 Compiling src/erc20.rs to ELF..."
	RUSTFLAGS="-C link-arg=-T/Users/alonmuroch/Projects/rust-vm/crates/examples/linker.ld" \
		cargo build --release \
			--target riscv32imac-unknown-none-elf \
			--manifest-path Cargo.toml \
			--bin erc20 \
			--features binaries
	@echo "🎯 Compiling src/multi_func.rs to ELF..."
	RUSTFLAGS="-C link-arg=-T/Users/alonmuroch/Projects/rust-vm/crates/examples/linker.ld" \
		cargo build --release \
			--target riscv32imac-unknown-none-elf \
			--manifest-path Cargo.toml \
			--bin multi_func \
			--features binaries
	@echo "🎯 Compiling src/simple.rs to ELF..."
	RUSTFLAGS="-C link-arg=-T/Users/alonmuroch/Projects/rust-vm/crates/examples/linker.ld" \
		cargo build --release \
			--target riscv32imac-unknown-none-elf \
			--manifest-path Cargo.toml \
			--bin simple \
			--features binaries
	@echo "🎯 Compiling src/storage.rs to ELF..."
	RUSTFLAGS="-C link-arg=-T/Users/alonmuroch/Projects/rust-vm/crates/examples/linker.ld" \
		cargo build --release \
			--target riscv32imac-unknown-none-elf \
			--manifest-path Cargo.toml \
			--bin storage \
			--features binaries
	@echo "✅ All example programs built successfully!"

list:
	@echo "📝 Available example programs:"
	@echo " - call_program"
	@echo " - erc20"
	@echo " - multi_func"
	@echo " - simple"
	@echo " - storage"

clean:
	@echo "🧹 Cleaning example programs..."
	cargo clean
	@echo "✅ Clean complete!"
