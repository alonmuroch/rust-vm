BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

all: list $(addprefix $(BIN_DIR)/, $(ELF))

list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

LINKER_SCRIPT := $(abspath linker.ld)
$(BIN_DIR)/%.elf: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF..."
	RUSTFLAGS="-C link-arg=-T$(LINKER_SCRIPT)" \
	cargo build --release \
		--target $(TARGET) \
		--manifest-path Cargo.toml \
		--bin $*

clean:
	rm -rf $(BIN_DIR)
