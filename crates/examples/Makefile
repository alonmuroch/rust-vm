BIN_DIR := bin
SRC := $(notdir $(wildcard src/*.rs))
ELF := $(SRC:.rs=.elf)

TARGET := riscv32imac-unknown-none-elf
LINKER_SCRIPT := linker.ld
BUILD_SCRIPT := ../compiler/scripts/compile.sh

.PRECIOUS: $(addprefix $(BIN_DIR)/, $(ELF))

all: list $(addprefix $(BIN_DIR)/, $(ELF))

list:
	@echo "📝 Compiling these source files:"
	@$(foreach file,$(SRC), echo " - $(file)";)

$(BIN_DIR)/%.elf: src/%.rs
	@mkdir -p $(BIN_DIR)
	@echo "🎯 Compiling $< to ELF via compile.sh"
	@sh $(BUILD_SCRIPT) $< $(BIN_DIR)/$*.elf $(LINKER_SCRIPT) $(TARGET)

clean:
	rm -rf $(BIN_DIR)
