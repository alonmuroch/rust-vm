#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SRC=$1
OUT=$2
LINKER=$3  # just the file name
TARGET=$4

LINKER_PATH="$SCRIPT_DIR/$LINKER"
PROGRAM_CRATE="$SCRIPT_DIR/../../../program"

# === Build libprogram ===
echo "📦 Building 'program' crate with target: $TARGET"
cargo build -p program --target "$TARGET"

# === Locate the generated .rlib ===
RLIB=$(find "$SCRIPT_DIR/../../../target/$TARGET/debug/deps" -name 'libprogram-*.rlib' | head -n 1)

if [[ -z "$RLIB" ]]; then
  echo "❌ Could not find libprogram rlib for target $TARGET"
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
  --emit=obj \
  -o "$OUT" \
  "$SRC"

echo "✅ Success: built $OUT"
