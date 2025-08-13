# Rust VM Examples - Build System
PROJECT_NAME := Rust VM Examples
VERSION := 1.0.0

BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)
ABI := $(SRC:.rs=.abi.json)
# Generate client code for each ABI
CLIENTS := $(ABI:.abi.json=_abi.rs)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

all: header list abi generate_clients $(addprefix $(BIN_DIR)/, $(ELF))

header:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  $(PROJECT_NAME) - v$(VERSION)"
	@echo "  Building RISC-V Programs with ABI Generation"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo ""

list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

abi: clean-abi $(addprefix $(BIN_DIR)/, $(ABI))
	@echo "📋 Generated ABIs for all programs"

clean-abi:
	@echo "🧹 Cleaning existing ABI files..."
	@rm -f $(addprefix $(BIN_DIR)/, $(ABI))

LINKER_SCRIPT := $(abspath linker.ld)

# General rule for all ELF files
$(BIN_DIR)/%.elf: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF..."
	RUSTFLAGS="-C link-arg=-T$(LINKER_SCRIPT)" \
	cargo build --release \
		--target $(TARGET) \
		--manifest-path Cargo.toml \
		--bin $* \
		--features binaries
	@cp ../../target/$(TARGET)/release/$* $(BIN_DIR)/$*.elf

$(BIN_DIR)/%.abi.json: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "📋 Generating ABI for $<..."
	@cd ../compiler && cargo run --bin abi_generator -- ../examples/$< ../examples/$@

generate_clients: $(addprefix $(BIN_DIR)/, $(CLIENTS))
	@echo "🔧 Generated client code for all ABIs"

# Rule to generate client code from each ABI
$(BIN_DIR)/%_abi.rs: $(BIN_DIR)/%.abi.json
	@mkdir -p $(BIN_DIR)
	@echo "🔧 Generating ABI client code for $*..."
	@CONTRACT_NAME=$$(echo $* | awk '{print toupper(substr($$0,1,1)) substr($$0,2)}')Contract; \
	cd ../compiler && cargo run -q --bin abi_codegen -- ../examples/bin/$*.abi.json ../examples/bin/$*_abi.rs $$CONTRACT_NAME 2>/dev/null || true

clean:
	@echo "🧹 Cleaning $(PROJECT_NAME) build artifacts..."
	@rm -rf $(BIN_DIR)
	@rm -rf src/generated
	@echo "✅ Clean complete"

help:
	@echo "$(PROJECT_NAME) - v$(VERSION)"
	@echo ""
	@echo "Available targets:"
	@echo "  make all              - Build all programs, generate ABIs and client code"
	@echo "  make abi              - Generate ABI files for all programs"
	@echo "  make generate_clients - Generate client code from all ABIs"
	@echo "  make clean            - Remove all build artifacts"
	@echo "  make help             - Show this help message"
	@echo ""
	@echo "Programs to build:"
	@$(foreach file,$(SRC), echo "  • $(file:.rs=)";)

.PHONY: all header list abi clean-abi generate_clients clean help
