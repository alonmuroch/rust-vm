#!/bin/bash
set -e

# === Resolve script location ===
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# === Arguments ===
SRC=$1         # Path to .rs file (absolute or relative to project root)
OUT=$2         # Output .elf path
RLIB=$3        # Path to libprogram-*.rlib
LINKER=$4      # Just the filename, e.g., linker.ld
TARGET=$5      # e.g., riscv32imac-unknown-none-elf

# === Full path to linker.ld (assumed to live next to this script) ===
LINKER_PATH="$SCRIPT_DIR/$LINKER"

# === Compile ===
echo "🔧 Compiling:"
echo "    SRC:    $SRC"
echo "    OUT:    $OUT"
echo "    RLIB:   $RLIB"
echo "    LINKER: $LINKER_PATH"
echo "    TARGET: $TARGET"
echo ""

rustc \
  --target="$TARGET" \
  -C opt-level=2 \
  -C panic=abort \
  --crate-type=bin \
  -C link-arg=-T"$LINKER_PATH" \
  -L "$(dirname "$RLIB")" \
  --extern program="$RLIB" \
  -o "$OUT" \
  "$SRC"

echo "✅ Success: $OUT"
