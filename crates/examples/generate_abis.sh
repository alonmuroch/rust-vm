#!/bin/bash

# Generate ABIs for all example programs
echo "🔧 Generating ABIs for all example programs..."

# Create bin directory if it doesn't exist
mkdir -p bin

# Clean existing ABI files
echo "🧹 Cleaning existing ABI files..."
rm -f bin/*.abi.json

# Dynamically find all Rust source files
echo "📋 Discovering Rust source files..."
programs=($(find src -name "*.rs" -exec basename {} .rs \;))
echo "Found programs: ${programs[*]}"

for program in "${programs[@]}"; do
    echo "📋 Generating ABI for $program.rs..."
    cd ../compiler && cargo run --bin abi_generator -- "../examples/src/${program}.rs" "../examples/bin/${program}.abi.json" && cd ../examples
done

echo "✅ ABI generation completed!"
echo "📁 Generated ABIs:"
ls -la bin/*.abi.json
