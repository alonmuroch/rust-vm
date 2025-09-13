use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod log_parser;
mod elf_instructions;
mod comparison;

use log_parser::VerboseLog;
use elf_instructions::ElfInstruction;
use comparison::ComparisonResult;

/// Binary comparison tool for comparing verbose VM logs with actual ELF binaries
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the folder containing ELF binaries
    #[arg(short, long)]
    binaries_folder: PathBuf,

    /// Path to the verbose log file from program_exec
    #[arg(short, long)]
    log_file: PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Show only differences (skip matching instructions)
    #[arg(short, long)]
    diff_only: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    println!("{}", "Binary Comparison Tool v0.1.0".bold().blue());
    println!("{}", "=====================================".blue());
    println!();

    // Validate inputs
    if !args.binaries_folder.exists() {
        anyhow::bail!("Binaries folder does not exist: {:?}", args.binaries_folder);
    }

    if !args.log_file.exists() {
        anyhow::bail!("Log file does not exist: {:?}", args.log_file);
    }

    // Read and parse the verbose log
    println!("{} {}", "📖".bold(), "Reading verbose log file...".cyan());
    let log_content = fs::read_to_string(&args.log_file)
        .context("Failed to read log file")?;
    
    let verbose_logs = log_parser::parse_verbose_log(&log_content)?;
    println!("  Found {} test cases in log", verbose_logs.len());

    // Process each test case
    let mut all_results = HashMap::new();
    
    for log in verbose_logs {
        println!();
        println!("{} Processing: {}", "🔍".bold(), log.test_name.yellow());
        
        // Print address mappings if any
        if !log.address_mappings.is_empty() {
            println!("  📍 Address mappings found: {}", log.address_mappings.len());
            for (addr, binary) in &log.address_mappings {
                println!("    {} -> {}", addr, binary);
            }
        } else {
            println!("  ⚠️  No address mappings found");
        }
        
        // Process each execution segment
        if !log.execution_segments.is_empty() {
            println!("  📊 Found {} execution segments", log.execution_segments.len());
            
            for segment in &log.execution_segments {
                println!();
                println!("  Segment: {} ({})", segment.address, segment.binary_name);
                println!("    Instructions: {} to {}", segment.start_index, segment.end_index);
                
                // Find the binary for this segment
                let binary_path = args.binaries_folder.join(format!("{}.elf", segment.binary_name));
                
                if binary_path.exists() {
                    // Decode ELF instructions for this binary
                    let elf_instructions = elf_instructions::decode_elf(&binary_path)?;
                    
                    // Get the instructions for this segment
                    let segment_instructions: Vec<_> = log.instructions[segment.start_index..=segment.end_index].to_vec();
                    
                    // Compare instructions
                    let comparison_result = comparison::compare_instructions(
                        &segment_instructions,
                        &elf_instructions,
                    );
                    
                    // Print summary for this segment
                    let segment_name = format!("{} - {}", log.test_name, segment.binary_name);
                    print_test_summary(&segment_name, &comparison_result, args.diff_only);
                    
                    all_results.insert(segment_name, comparison_result);
                } else {
                    println!("    ⚠️  Binary not found: {}", binary_path.display());
                }
            }
        } else {
            // Fallback to old behavior if no segments detected
            let binary_path = find_binary_for_test(&args.binaries_folder, &log.test_name)?;
            
            if let Some(binary_path) = binary_path {
                println!("  Binary: {}", binary_path.display());
                
                // Decode ELF instructions
                let elf_instructions = elf_instructions::decode_elf(&binary_path)?;
                println!("  ELF instructions: {}", elf_instructions.len());
                
                // Compare instructions
                let comparison_result = comparison::compare_instructions(
                    &log.instructions,
                    &elf_instructions,
                );
                
                // Print summary for this test
                print_test_summary(&log.test_name, &comparison_result, args.diff_only);
                
                all_results.insert(log.test_name.clone(), comparison_result);
            } else {
                println!("  ⚠️  No binary found for test: {}", log.test_name);
            }
        }
    }

    // Print overall summary
    print_overall_summary(&all_results);

    // Export results if needed
    if args.format == "json" {
        let json_output = serde_json::to_string_pretty(&all_results)?;
        let output_path = args.log_file.with_extension("comparison.json");
        fs::write(&output_path, json_output)?;
        println!();
        println!("📝 Results exported to: {}", output_path.display());
    }

    Ok(())
}

fn find_binary_for_test(binaries_folder: &Path, test_name: &str) -> Result<Option<PathBuf>> {
    // Extract binary name from test name
    // Test names are usually like "Test 1: simple" or "Test 7: ERC20 Token"
    let binary_name = extract_binary_name(test_name);
    
    // Look for the binary file
    let binary_path = binaries_folder.join(&binary_name);
    
    if binary_path.exists() {
        Ok(Some(binary_path))
    } else {
        // Try with common extensions
        for ext in &["", ".elf", ".bin"] {
            let path_with_ext = binaries_folder.join(format!("{}{}", binary_name, ext));
            if path_with_ext.exists() {
                return Ok(Some(path_with_ext));
            }
        }
        Ok(None)
    }
}

fn extract_binary_name(test_name: &str) -> String {
    // Map test names to binary names
    match test_name.to_lowercase().as_str() {
        "erc20" => "erc20".to_string(),
        "call program" => "call_program".to_string(),
        "account create (storage)" => "storage".to_string(),
        "account create (simple)" => "simple".to_string(),
        "multi function (simple)" => "multi_func".to_string(),
        "allocator demo" => "allocator_demo".to_string(),
        "lib_import - sha256 of 'hello world'" => "lib_import".to_string(),
        "logging demo" => "logging".to_string(),
        _ => {
            // Fallback: remove special chars and convert spaces to underscores
            test_name.to_lowercase()
                .replace("(", "")
                .replace(")", "")
                .replace("-", "")
                .replace("'", "")
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or(test_name)
                .to_string()
        }
    }
}

fn print_test_summary(test_name: &str, result: &ComparisonResult, diff_only: bool) {
    println!();
    println!("  {}", "Summary:".bold());
    println!("    Total instructions: {}", result.total_instructions);
    println!("    Matching: {} ({}%)", 
        result.matching_instructions.to_string().green(),
        ((result.matching_instructions as f64 / result.total_instructions as f64) * 100.0) as u32
    );
    
    if !result.differences.is_empty() {
        println!("    Differences: {} ({}%)", 
            result.differences.len().to_string().red(),
            ((result.differences.len() as f64 / result.total_instructions as f64) * 100.0) as u32
        );
        
        if !diff_only {
            println!();
            println!("    {}", "First 5 differences:".yellow());
            for (i, diff) in result.differences.iter().take(5).enumerate() {
                println!("      {}. PC: 0x{:08x}", i + 1, diff.pc);
                println!("         Log:  {}", diff.log_instruction);
                println!("         ELF:  {}", diff.elf_instruction);
            }
        }
    }
}

fn print_overall_summary(results: &HashMap<String, ComparisonResult>) {
    println!();
    println!("{}", "=====================================".blue());
    println!("{}", "Overall Summary".bold().green());
    println!("{}", "=====================================".blue());
    
    let mut total_instructions = 0;
    let mut total_matching = 0;
    let mut total_differences = 0;
    
    for (test_name, result) in results {
        total_instructions += result.total_instructions;
        total_matching += result.matching_instructions;
        total_differences += result.differences.len();
        
        let match_rate = (result.matching_instructions as f64 / result.total_instructions as f64) * 100.0;
        
        let status = if match_rate >= 100.0 {
            "✅ PERFECT".green()
        } else if match_rate >= 95.0 {
            "⚠️  GOOD".yellow()
        } else {
            "❌ NEEDS REVIEW".red()
        };
        
        println!("  {} - {}: {:.1}% match", status, test_name, match_rate);
    }
    
    println!();
    println!("  {}", "Totals:".bold());
    println!("    Instructions analyzed: {}", total_instructions);
    println!("    Matching: {} ({}%)", 
        total_matching,
        ((total_matching as f64 / total_instructions as f64) * 100.0) as u32
    );
    println!("    Differences: {} ({}%)", 
        total_differences,
        ((total_differences as f64 / total_instructions as f64) * 100.0) as u32
    );
}