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

# Show detailed instruction-by-instruction comparison
cargo run -- -b ../../crates/examples/bin -l logs.txt --detailed

# Export results as JSON
cargo run -- -b ../../crates/examples/bin -l logs.txt --format json
```

## Arguments

- `-b, --binaries-folder`: Path to folder containing ELF binaries
- `-l, --log-file`: Path to verbose log file from program_exec test
- `-f, --format`: Output format (text or json, default: text)
- `--detailed`: Show detailed instruction-by-instruction comparison with match status

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

### Standard Summary Mode
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

### Detailed Mode (--detailed)
```
call program - simple: Detailed Instruction Comparison
============================================================
   1 [✓] PC: 0x00000400 | addi x10, x0, 7
   2 [✓] PC: 0x00000402 | bgeu x10, x13, pc+166
   3 [✓] PC: 0x00000406 | lbu   x10, 1(x12)
   4 [✗] PC: 0x0000040a
        Log: lbu   x11, 2(x12) (0x0260c583)
        ELF: lbu   x11, 3(x12) (0x0360c583)
   5 [?] PC: 0x0000040e | lbu   x13, 3(x12) (log only)
============================================================
Summary: 3 matching, 1 different, 1 log-only (of 5 total)
```

Legend:
- `[✓]` - Instruction matches between log and ELF (green)
- `[✗]` - Different instruction at same PC (red/yellow)
- `[?]` - Instruction only in log, not in ELF (yellow)

## Integration with Tests

The tool uses centralized ELF binary definitions from `crates/examples/tests/elfs.rs`, ensuring consistency between test execution and binary comparison.