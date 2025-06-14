#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SRC=$1
OUT=$2
LINKER=$3  # just the file name
TARGET=$4

LINKER_PATH="$SCRIPT_DIR/$LINKER"
PROGRAM_CRATE="$SCRIPT_DIR/../../program"

# === Build only the library of the 'program' crate ===
echo "📦 Building 'program' crate (lib only) with target: $TARGET"
cargo build --manifest-path "$PROGRAM_CRATE/Cargo.toml" --target "$TARGET" --lib

# === Locate the generated .rlib ===
RLIB=$(find "$SCRIPT_DIR/../../../target/$TARGET/debug/deps" -type f -name 'libprogram-*.rlib' | sort | head -n 1)

if [[ -z "$RLIB" ]]; then
  echo "❌ Could not find libprogram .rlib for target $TARGET"
  exit 1
fi

echo "🔧 Compiling guest Rust program:"
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
  --emit=link \
  -o "$OUT" \
  "$SRC"

echo "✅ Success: built $OUT"
