use crate::elf_instructions::ElfInstruction;
use crate::log_parser::LogInstruction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub total_instructions: usize,
    pub matching_instructions: usize,
    pub differences: Vec<InstructionDifference>,
    pub log_only: Vec<LogInstruction>,
    pub elf_only: Vec<ElfInstruction>,
    pub instruction_stats: InstructionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionDifference {
    pub pc: u32,
    pub log_instruction: String,
    pub elf_instruction: String,
    pub raw_log: u32,
    pub raw_elf: u32,
    pub difference_type: DifferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceType {
    DifferentInstruction,
    DifferentEncoding,
    DifferentOperands,
    PcMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionStats {
    pub instruction_counts: HashMap<String, usize>,
    pub mismatch_by_type: HashMap<String, usize>,
}

/// Compare instructions from log and ELF
pub fn compare_instructions(
    log_instructions: &[LogInstruction],
    elf_instructions: &[ElfInstruction],
) -> ComparisonResult {
    let mut differences = Vec::new();
    let mut log_only = Vec::new();
    let mut elf_only = Vec::new();
    let mut matching = 0;
    let mut instruction_counts = HashMap::new();
    let mut mismatch_by_type = HashMap::new();
    
    // Create maps for efficient lookup
    let mut log_map: HashMap<u32, &LogInstruction> = HashMap::new();
    for inst in log_instructions {
        log_map.insert(inst.pc, inst);
    }
    
    let mut elf_map: HashMap<u32, &ElfInstruction> = HashMap::new();
    for inst in elf_instructions {
        elf_map.insert(inst.pc, inst);
    }
    
    // Compare log instructions against ELF
    for log_inst in log_instructions {
        // Track instruction type
        let inst_type = extract_instruction_type(&log_inst.decoded);
        *instruction_counts.entry(inst_type.clone()).or_insert(0) += 1;
        
        if let Some(elf_inst) = elf_map.get(&log_inst.pc) {
            // Both have instruction at this PC
            if instructions_match(log_inst, elf_inst) {
                matching += 1;
            } else {
                let diff_type = determine_difference_type(log_inst, elf_inst);
                
                // Track mismatch type
                *mismatch_by_type.entry(inst_type).or_insert(0) += 1;
                
                differences.push(InstructionDifference {
                    pc: log_inst.pc,
                    log_instruction: format_log_instruction(log_inst),
                    elf_instruction: format_elf_instruction(elf_inst),
                    raw_log: log_inst.raw_instruction,
                    raw_elf: elf_inst.raw,
                    difference_type: diff_type,
                });
            }
        } else {
            // Instruction only in log
            log_only.push(log_inst.clone());
        }
    }
    
    // Find instructions only in ELF
    for elf_inst in elf_instructions {
        if !log_map.contains_key(&elf_inst.pc) {
            elf_only.push(elf_inst.clone());
        }
    }
    
    let total = log_instructions.len().max(elf_instructions.len());
    
    ComparisonResult {
        total_instructions: total,
        matching_instructions: matching,
        differences,
        log_only,
        elf_only,
        instruction_stats: InstructionStats {
            instruction_counts,
            mismatch_by_type,
        },
    }
}

/// Check if two instructions match
fn instructions_match(log: &LogInstruction, elf: &ElfInstruction) -> bool {
    // First check raw instruction bytes
    if log.raw_instruction == elf.raw {
        return true;
    }
    
    // If raw doesn't match, check if decoded instructions are semantically equivalent
    normalize_instruction(&log.decoded) == normalize_instruction(&elf.decoded)
}

/// Normalize instruction string for comparison
fn normalize_instruction(inst: &str) -> String {
    // Remove extra whitespace and normalize format
    let normalized = inst.trim().to_lowercase();
    
    // Remove instruction encoding in parentheses if present
    if let Some(paren_pos) = normalized.find("(0x") {
        normalized[..paren_pos].trim().to_string()
    } else {
        normalized
    }
}

/// Extract instruction type from decoded string
fn extract_instruction_type(decoded: &str) -> String {
    let parts: Vec<&str> = decoded.split_whitespace().collect();
    if !parts.is_empty() {
        parts[0].to_lowercase()
    } else {
        "unknown".to_string()
    }
}

/// Determine the type of difference between instructions
fn determine_difference_type(log: &LogInstruction, elf: &ElfInstruction) -> DifferenceType {
    if log.pc != elf.pc {
        return DifferenceType::PcMismatch;
    }
    
    let log_type = extract_instruction_type(&log.decoded);
    let elf_type = extract_instruction_type(&elf.decoded);
    
    if log_type != elf_type {
        return DifferenceType::DifferentInstruction;
    }
    
    if log.raw_instruction != elf.raw {
        return DifferenceType::DifferentEncoding;
    }
    
    DifferenceType::DifferentOperands
}

/// Format log instruction for display
fn format_log_instruction(inst: &LogInstruction) -> String {
    if inst.decoded.is_empty() {
        format!("0x{:08x}", inst.raw_instruction)
    } else {
        inst.decoded.clone()
    }
}

/// Format ELF instruction for display
fn format_elf_instruction(inst: &ElfInstruction) -> String {
    if inst.decoded.is_empty() {
        format!("0x{:08x}", inst.raw)
    } else {
        inst.decoded.clone()
    }
}

/// Generate a summary report of the comparison
pub fn generate_summary_report(results: &HashMap<String, ComparisonResult>) -> String {
    let mut report = String::new();
    
    report.push_str("Binary Comparison Summary Report\n");
    report.push_str("=================================\n\n");
    
    for (test_name, result) in results {
        report.push_str(&format!("Test: {}\n", test_name));
        report.push_str(&format!("  Total Instructions: {}\n", result.total_instructions));
        report.push_str(&format!("  Matching: {} ({:.1}%)\n", 
            result.matching_instructions,
            (result.matching_instructions as f64 / result.total_instructions as f64) * 100.0
        ));
        
        if !result.differences.is_empty() {
            report.push_str(&format!("  Differences: {}\n", result.differences.len()));
            
            // Group differences by type
            let mut by_type: HashMap<String, usize> = HashMap::new();
            for diff in &result.differences {
                let type_name = match diff.difference_type {
                    DifferenceType::DifferentInstruction => "Different Instruction",
                    DifferenceType::DifferentEncoding => "Different Encoding",
                    DifferenceType::DifferentOperands => "Different Operands",
                    DifferenceType::PcMismatch => "PC Mismatch",
                };
                *by_type.entry(type_name.to_string()).or_insert(0) += 1;
            }
            
            for (diff_type, count) in by_type {
                report.push_str(&format!("    - {}: {}\n", diff_type, count));
            }
        }
        
        if !result.log_only.is_empty() {
            report.push_str(&format!("  Instructions only in log: {}\n", result.log_only.len()));
        }
        
        if !result.elf_only.is_empty() {
            report.push_str(&format!("  Instructions only in ELF: {}\n", result.elf_only.len()));
        }
        
        report.push_str("\n");
    }
    
    report
}