# Binary Comparison Tool

A CLI utility for comparing verbose VM execution logs with actual ELF binaries to identify instruction-level differences.

## Purpose

This tool helps debug and verify the VM implementation by comparing:
- Instructions executed by the VM (from verbose logs)
- Instructions in the original ELF binaries
- Identifying mismatches in instruction decoding or execution

## Usage

```bash
# Basic usage
cargo run -- -b <binaries_folder> -l <log_file>

# Example
cargo run -- -b ../../crates/examples/bin -l /Users/alonmuroch/Desktop/logs.txt

# Show only differences
cargo run -- -b ../../crates/examples/bin -l logs.txt --diff-only

# Export results as JSON
cargo run -- -b ../../crates/examples/bin -l logs.txt --format json
```

## Arguments

- `-b, --binaries-folder`: Path to folder containing ELF binaries
- `-l, --log-file`: Path to verbose log file from program_exec test
- `-f, --format`: Output format (text or json, default: text)
- `-d, --diff-only`: Show only differences, skip matching instructions

## How It Works

1. **Parse Verbose Logs**: Extracts instruction sequences from VM execution logs
2. **Decode ELF Binaries**: Reads and decodes RISC-V instructions from ELF files
3. **Compare Instructions**: Matches instructions by PC address and compares:
   - Raw instruction bytes
   - Decoded instruction text
   - Instruction types and operands
4. **Generate Report**: Produces a summary showing:
   - Match percentage
   - Types of differences
   - Instruction statistics

## Output

The tool provides:
- Per-test comparison results
- Overall summary statistics
- Detailed difference analysis
- Optional JSON export for further processing

## Example Output

```
Binary Comparison Tool v0.1.0
=====================================

📖 Reading verbose log file...
  Found 8 test cases in log

🔍 Processing: Test 1: simple
  Binary: ../../crates/examples/bin/simple
  ELF instructions: 1024
  
  Summary:
    Total instructions: 1024
    Matching: 1020 (99%)
    Differences: 4 (1%)
    
    First 5 differences:
      1. PC: 0x00001000
         Log:  addi x2, x2, -48
         ELF:  addi sp, sp, -48

=====================================
Overall Summary
=====================================
  ✅ PERFECT - Test 1: simple: 100.0% match
  ⚠️  GOOD - Test 2: storage: 98.5% match
  
  Totals:
    Instructions analyzed: 8192
    Matching: 8100 (98%)
    Differences: 92 (2%)
```

## Integration with Tests

The tool uses centralized ELF binary definitions from `crates/examples/tests/elfs.rs`, ensuring consistency between test execution and binary comparison.