# Binary Comparison Tool Makefile

.PHONY: all build clean release help

# Default target
all: build

# Build in debug mode
build:
	@echo "🔨 Building binary comparison tool (debug)..."
	@cargo build
	@echo "✅ Binary comparison tool built successfully"
	@echo "   Binary location: target/debug/binary_comparison"

# Build in release mode (optimized)
release:
	@echo "🚀 Building binary comparison tool (release)..."
	@cargo build --release
	@echo "✅ Binary comparison tool built successfully"
	@echo "   Binary location: target/release/binary_comparison"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning binary comparison build artifacts..."
	@cargo clean
	@echo "✅ Clean complete"

# Run the tool with example arguments
run-example: release
	@echo "📊 Running binary comparison example..."
	@echo "   Note: This requires a log file at /Users/alonmuroch/Desktop/logs.txt"
	@./target/release/binary_comparison \
		-b ../../crates/examples/bin \
		-l /Users/alonmuroch/Desktop/logs.txt \
		|| echo "⚠️  Example run failed. Make sure you have a log file from running tests with verbose output."

# Show help
help:
	@echo "Binary Comparison Tool Makefile"
	@echo "==============================="
	@echo ""
	@echo "Available targets:"
	@echo "  make build       - Build the tool in debug mode"
	@echo "  make release     - Build the tool in release mode (optimized)"
	@echo "  make clean       - Remove build artifacts"
	@echo "  make run-example - Run an example comparison (requires log file)"
	@echo "  make help        - Show this help message"
	@echo ""
	@echo "Usage:"
	@echo "  After building, run the tool with:"
	@echo "  ./target/release/binary_comparison -b <binaries_folder> -l <log_file>"