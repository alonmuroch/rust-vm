BIN_DIR := bin
SRC := $(wildcard *.rs)
OBJ := $(SRC:.rs=.o)
BIN := $(SRC:.rs=.bin)

TARGET := riscv32imac-unknown-none-elf
RUSTC := rustc
OBJCOPY := $(shell command -v rust-objcopy || command -v llvm-objcopy || command -v riscv64-unknown-elf-objcopy)

# Prevent Make from deleting .o files after .bin is created
.PRECIOUS: $(addprefix $(BIN_DIR)/, $(OBJ))

all: list $(addprefix $(BIN_DIR)/, $(BIN))

# List which files are being compiled
list:
	@echo "Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

# Compile .rs to .o
$(BIN_DIR)/%.o: %.rs
	@mkdir -p $(BIN_DIR)
	$(RUSTC) \
		--target=$(TARGET) \
		-C opt-level=2 \
		-C panic=abort \
		--crate-type=staticlib \
		--emit=obj \
		--out-dir=$(BIN_DIR) \
		$<

# Convert .o to .bin using only .text section
$(BIN_DIR)/%.bin: $(BIN_DIR)/%.o
	@echo "Using objcopy: $(OBJCOPY)"
	$(OBJCOPY) --only-section=.text.main -O binary $< $@

# Clean build outputs
clean:
	rm -rf $(BIN_DIR)
