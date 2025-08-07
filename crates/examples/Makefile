BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)
ABI := $(SRC:.rs=.abi.json)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

all: list $(addprefix $(BIN_DIR)/, $(ELF)) abi

list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

abi: clean-abi $(addprefix $(BIN_DIR)/, $(ABI))
	@echo "📋 Generated ABIs for all programs"

clean-abi:
	@echo "🧹 Cleaning existing ABI files..."
	@rm -f $(addprefix $(BIN_DIR)/, $(ABI))

LINKER_SCRIPT := $(abspath linker.ld)
$(BIN_DIR)/%.elf: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF..."
	RUSTFLAGS="-C link-arg=-T$(LINKER_SCRIPT)" \
	cargo build --release \
		--target $(TARGET) \
		--manifest-path Cargo.toml \
		--bin $* \
		--features binaries

$(BIN_DIR)/%.abi.json: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "📋 Generating ABI for $<..."
	@cd ../compiler && cargo run --bin abi_generator -- ../examples/$< ../examples/$@

clean:
	rm -rf $(BIN_DIR)
