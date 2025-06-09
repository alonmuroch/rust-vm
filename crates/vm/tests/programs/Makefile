BIN_DIR := bin
SRC := $(wildcard *.rs)
OBJ := $(SRC:.rs=.o)
BIN := $(SRC:.rs=.bin)

TARGET := riscv32imac-unknown-none-elf
RUSTC := rustc
OBJCOPY := $(shell command -v rust-objcopy || command -v llvm-objcopy || command -v riscv64-unknown-elf-objcopy)

# Find the compiled libprogram .rlib file
PROGRAM_RLIB := $(shell find ../../../../target/$(TARGET)/debug/deps -name 'libprogram-*.rlib' | head -n 1)

# Prevent Make from deleting intermediate .o files
.PRECIOUS: $(addprefix $(BIN_DIR)/, $(OBJ))

# Default target
all: build_program list $(addprefix $(BIN_DIR)/, $(BIN))

# Build the program crate first (needed for --extern)
build_program:
	@echo "📦 Building program crate..."
	cargo build -p program --target=$(TARGET)

# List which Rust files are being compiled
list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

# Compile .rs to .o
$(BIN_DIR)/%.o: %.rs
	@mkdir -p $(BIN_DIR)
	@echo "🔗 Linking against: $(PROGRAM_RLIB)"
	$(RUSTC) \
		--target=$(TARGET) \
		-C opt-level=2 \
		-C panic=abort \
		--crate-type=bin \
		--emit=obj \
		-L ../../../../target/$(TARGET)/debug/deps \
		--extern program=$(PROGRAM_RLIB) \
		-o $@ \
		$<

# Convert .o to .bin by extracting the .text section
$(BIN_DIR)/%.bin: $(BIN_DIR)/%.o
	@echo "📦 Extracting .text section to raw binary..."
	$(OBJCOPY) --only-section=.text.entrypoint -O binary $< $@

# Clean build outputs
clean:
	rm -rf $(BIN_DIR)
