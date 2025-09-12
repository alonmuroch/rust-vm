use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerboseLog {
    pub test_name: String,
    pub instructions: Vec<LogInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogInstruction {
    pub pc: u32,
    pub raw_instruction: u32,
    pub decoded: String,
    pub registers_before: Option<RegisterState>,
    pub registers_after: Option<RegisterState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterState {
    pub registers: Vec<u32>,
}

/// Parse the verbose log file and extract instruction sequences for each test
pub fn parse_verbose_log(content: &str) -> Result<Vec<VerboseLog>> {
    let mut logs = Vec::new();
    let mut current_test: Option<VerboseLog> = None;
    let mut current_instructions = Vec::new();
    
    // Regex patterns for parsing
    let test_header_re = Regex::new(r"#### Running test case: (.+) ####")?;
    
    // Pattern for the format: PC = 0x00000400, Bytes = [13 01 01 bd], Instr = addi x2, x2, -1072
    let instruction_line_re = Regex::new(r"PC = 0x([0-9a-fA-F]+), Bytes = \[([0-9a-fA-F ]+)\], Instr = (.+)")?;
    
    // Alternative patterns for other formats
    let executing_re = Regex::new(r"Executing \[0x([0-9a-fA-F]+)\]: (.+)")?;
    let compact_re = Regex::new(r"\[0x([0-9a-fA-F]+)\] 0x([0-9a-fA-F]+): (.+)")?;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Check for test case header
        if let Some(captures) = test_header_re.captures(line) {
            // Save previous test if any
            if let Some(mut test) = current_test.take() {
                test.instructions = current_instructions.clone();
                logs.push(test);
                current_instructions.clear();
            }
            
            // Start new test
            let test_name = captures[1].to_string();
            current_test = Some(VerboseLog {
                test_name,
                instructions: Vec::new(),
            });
        }
        
        // Parse instruction in the format: PC = 0x00000400, Bytes = [13 01 01 bd], Instr = addi x2, x2, -1072
        if let Some(captures) = instruction_line_re.captures(line) {
            let pc = u32::from_str_radix(&captures[1], 16)?;
            let bytes_str = &captures[2];
            let decoded = captures[3].to_string();
            
            // Parse the bytes (they're in hex, space-separated, little-endian)
            let bytes: Vec<u8> = bytes_str
                .split_whitespace()
                .filter_map(|b| u8::from_str_radix(b, 16).ok())
                .collect();
            
            // Convert bytes to instruction (little-endian)
            let raw_instruction = if bytes.len() >= 4 {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            } else if bytes.len() == 2 {
                u16::from_le_bytes([bytes[0], bytes[1]]) as u32
            } else {
                0
            };
            
            current_instructions.push(LogInstruction {
                pc,
                raw_instruction,
                decoded,
                registers_before: None,
                registers_after: None,
            });
            i += 1;
            continue;
        }
        
        // Parse instruction in executing format
        if let Some(captures) = executing_re.captures(line) {
            let pc = u32::from_str_radix(&captures[1], 16)?;
            let decoded = captures[2].to_string();
            
            // Try to extract the raw instruction from the decoded string
            // Format might be like "addi x2, x2, -48 (0x13010fd0)"
            let raw_instruction = if let Some(paren_pos) = decoded.rfind("(0x") {
                let hex_start = paren_pos + 3;
                if let Some(paren_end) = decoded[hex_start..].find(')') {
                    let hex = &decoded[hex_start..hex_start + paren_end];
                    u32::from_str_radix(hex, 16).unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };
            
            current_instructions.push(LogInstruction {
                pc,
                raw_instruction,
                decoded: decoded.clone(),
                registers_before: None,
                registers_after: None,
            });
        }
        
        // Parse compact format [PC] INSTRUCTION: DECODED
        if let Some(captures) = compact_re.captures(line) {
            let pc = u32::from_str_radix(&captures[1], 16)?;
            let raw_instruction = u32::from_str_radix(&captures[2], 16)?;
            let decoded = captures[3].to_string();
            
            current_instructions.push(LogInstruction {
                pc,
                raw_instruction,
                decoded,
                registers_before: None,
                registers_after: None,
            });
        }
        
        i += 1;
    }
    
    // Save the last test
    if let Some(mut test) = current_test {
        test.instructions = current_instructions;
        logs.push(test);
    }
    
    Ok(logs)
}

/// Parse register state from log line
pub fn parse_register_state(line: &str) -> Option<RegisterState> {
    // Example: "Registers: x0=0x00000000 x1=0x00001000 x2=0x00002000 ..."
    if !line.contains("Registers:") && !line.contains("x0=") {
        return None;
    }
    
    let mut registers = vec![0u32; 32];
    let reg_re = Regex::new(r"x(\d+)=0x([0-9a-fA-F]+)").ok()?;
    
    for captures in reg_re.captures_iter(line) {
        let reg_num: usize = captures[1].parse().ok()?;
        let value = u32::from_str_radix(&captures[2], 16).ok()?;
        
        if reg_num < 32 {
            registers[reg_num] = value;
        }
    }
    
    Some(RegisterState { registers })
}