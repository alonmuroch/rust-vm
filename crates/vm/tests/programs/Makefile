BIN_DIR := bin
SRC := $(wildcard *.rs)
ELF := $(SRC:.rs=.elf)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld
BUILD_SCRIPT := ../../../compiler/scripts/compile.sh

PROGRAM_RLIB := $(shell find ../../../../target/$(TARGET)/debug/deps -name 'libprogram-*.rlib' | head -n 1)

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

# Default target
all: build_program list $(addprefix $(BIN_DIR)/, $(ELF))

# Build the dependency crate first
build_program:
	@echo "📦 Building program crate..."
	cargo build -p program --target=$(TARGET)

# List source files
list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

# Compile .rs to .elf using external shell script
$(BIN_DIR)/%.elf: %.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF via compile.sh"
	@sh $(BUILD_SCRIPT) $< $(BIN_DIR)/$*.elf $(PROGRAM_RLIB) $(LINKER_SCRIPT) $(TARGET)

# Clean output
clean:
	rm -rf $(BIN_DIR)
