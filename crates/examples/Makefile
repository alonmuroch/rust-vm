# Rust VM Examples - Build System
PROJECT_NAME := Rust VM Examples
VERSION := 1.0.0
CARGO_NIGHTLY ?= cargo +nightly-aarch64-apple-darwin
AVM32 := $(CARGO_NIGHTLY) run -p compiler --bin avm32 --

BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)
ABI := $(SRC:.rs=.abi.json)
# Generate client code for each ABI
CLIENTS := $(ABI:.abi.json=_abi.rs)

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

# General rule for all ELF files
$(BIN_DIR)/%.elf: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF..."
	@$(AVM32) all --bin $* --out-dir $(BIN_DIR)

$(BIN_DIR)/%.abi.json: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "📋 Generating ABI for $<..."
	@$(AVM32) abi --bin $* --src $< --out $(BIN_DIR)/$*.abi.json

generate_clients: $(addprefix $(BIN_DIR)/, $(CLIENTS))
	@echo "🔧 Generated client code for all ABIs"

# Rule to generate client code from each ABI
$(BIN_DIR)/%_abi.rs: $(BIN_DIR)/%.abi.json
	@mkdir -p $(BIN_DIR)
	@echo "🔧 Generating ABI client code for $*..."
	@$(AVM32) client --abi $(BIN_DIR)/$*.abi.json --out $(BIN_DIR)/$*_abi.rs

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
