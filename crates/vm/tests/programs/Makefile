BIN_DIR := bin
SRC := $(wildcard *.rs)
ELF := $(SRC:.rs=.elf)
BIN := $(SRC:.rs=.bin)

TARGET := riscv32imac-unknown-none-elf
RUSTC := rustc
OBJCOPY := $(shell command -v rust-objcopy || command -v llvm-objcopy || command -v riscv64-unknown-elf-objcopy)

# Dynamically resolve contract_std .rlib path
CONTRACT_STD_RLIB := $(shell find ../../../../target/$(TARGET)/debug/deps -name 'libcontract_std-*.rlib' 2>/dev/null)

# Keep ELF outputs
.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

all: list contract_std $(addprefix $(BIN_DIR)/, $(BIN))

# Show which files are being compiled
list:
	@echo "Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

# Ensure contract_std is built before using it
contract_std:
ifeq ($(strip $(CONTRACT_STD_RLIB)),)
	@echo "📦 Building contract_std..."
	cargo build -p contract_std --target=$(TARGET)
	$(eval CONTRACT_STD_RLIB := $(shell find ../../../../target/$(TARGET)/debug/deps -name 'libcontract_std-*.rlib' 2>/dev/null))
endif

# Compile .rs to .elf with --extern contract_std
$(BIN_DIR)/%.elf: %.rs
	@mkdir -p $(BIN_DIR)
	@echo "🔗 Linking against: $(CONTRACT_STD_RLIB)"
	$(RUSTC) \
		--target=$(TARGET) \
		-C opt-level=2 \
		-C panic=abort \
		-L ../../../../target/$(TARGET)/debug/deps \
		--extern contract_std=$(CONTRACT_STD_RLIB) \
		--crate-type=bin \
		--emit=link \
		-o $@ \
		$<

# Convert .elf to .bin
$(BIN_DIR)/%.bin: $(BIN_DIR)/%.elf
	@echo "📦 Extracting .text section to raw binary..."
	$(OBJCOPY) --only-section=.text -O binary $< $@

clean:
	rm -rf $(BIN_DIR)
