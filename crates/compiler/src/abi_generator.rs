use std::fs;
use std::path::Path;
use crate::abi::{ContractAbi, FunctionAbi, EventAbi, EventParam, ParamType};

/// ABI Generator that analyzes Rust source code to extract function and event definitions
pub struct AbiGenerator {
    source_code: String,
    abi: ContractAbi,
}

impl AbiGenerator {
    /// Create a new ABI generator from source code
    pub fn new(source_code: String) -> Self {
        Self {
            source_code,
            abi: ContractAbi::new(),
        }
    }

    /// Generate ABI from source code
    pub fn generate(&mut self) -> ContractAbi {
        self.extract_events();
        self.extract_functions();
        self.abi.clone()
    }

    /// Extract event definitions from source code
    fn extract_events(&mut self) {
        // Look for event! macro invocations
        let lines: Vec<&str> = self.source_code.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("event!(") {
                if let Some((event, consumed_lines)) = self.parse_event_macro_multi_line(&lines, i) {
                    self.abi.add_event(event);
                    i += consumed_lines;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    /// Parse an event! macro invocation that may span multiple lines
    fn parse_event_macro_multi_line(&self, lines: &[&str], start_line: usize) -> Option<(EventAbi, usize)> {
        let mut event_content = String::new();
        let mut brace_count = 0;
        let mut in_event = false;
        let mut consumed_lines = 0;
        
        for i in start_line..lines.len() {
            let line = lines[i].trim();
            consumed_lines += 1;
            
            if !in_event {
                if line.starts_with("event!(") {
                    in_event = true;
                    event_content.push_str(line);
                    // Count opening braces
                    brace_count += line.chars().filter(|&c| c == '{').count();
                    brace_count -= line.chars().filter(|&c| c == '}').count();
                }
            } else {
                event_content.push_str(line);
                // Count braces
                brace_count += line.chars().filter(|&c| c == '{').count();
                brace_count -= line.chars().filter(|&c| c == '}').count();
                
                // If we've closed all braces, we're done
                if brace_count == 0 && line.ends_with(");") {
                    break;
                }
            }
        }
        
        if !in_event {
            return None;
        }
        
        self.parse_event_macro(&event_content).map(|event| (event, consumed_lines))
    }

    /// Parse an event! macro invocation
    fn parse_event_macro(&self, line: &str) -> Option<EventAbi> {
        // Simple parsing for event! macro
        // Example: event!(Minted { caller => Address, amount => u32, });
        
        let trimmed = line.trim();
        if !trimmed.starts_with("event!(") || !trimmed.ends_with(");") {
            return None;
        }

        // Extract the content between event!( and );
        let content = &trimmed[7..trimmed.len()-2];
        
        // Find the event name (first word before {)
        let brace_pos = content.find('{')?;
        let event_name = content[..brace_pos].trim();
        
        // Extract field definitions
        let fields_content = &content[brace_pos+1..];
        let fields_content = fields_content.trim_end_matches('}');
        
        let mut inputs = Vec::new();
        let field_lines: Vec<&str> = fields_content.split(',').collect();
        
        for field_line in field_lines {
            let field_line = field_line.trim();
            if field_line.is_empty() {
                continue;
            }
            
            // Parse field: name => type
            if let Some((name, param_type)) = self.parse_event_field(field_line) {
                inputs.push(EventParam {
                    name: name.to_string(),
                    kind: param_type,
                    indexed: false, // Default to false for now
                });
            }
        }
        
        Some(EventAbi {
            name: event_name.to_string(),
            inputs,
        })
    }

    /// Parse an event field definition
    fn parse_event_field<'a>(&self, field: &'a str) -> Option<(&'a str, ParamType)> {
        // Format: field_name => type
        let parts: Vec<&str> = field.split("=>").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let name = parts[0].trim();
        let type_str = parts[1].trim();
        
        let param_type = self.parse_param_type(type_str)?;
        Some((name, param_type))
    }

    /// Parse a parameter type string
    fn parse_param_type(&self, type_str: &str) -> Option<ParamType> {
        match type_str {
            "Address" => Some(ParamType::Address),
            "u32" => Some(ParamType::Uint(32)),
            "u64" => Some(ParamType::Uint(64)),
            "u8" => Some(ParamType::Uint(8)),
            "bool" => Some(ParamType::Bool),
            "String" => Some(ParamType::String),
            "&[u8]" | "[u8]" => Some(ParamType::Bytes),
            _ => None,
        }
    }

    /// Extract function definitions from source code
    fn extract_functions(&mut self) {
        // Look for router patterns and function selectors
        let lines: Vec<&str> = self.source_code.lines().collect();
        
        let mut functions_to_add = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if line.contains("call.selector") {
                self.collect_functions_from_router(&lines, i, &mut functions_to_add);
            }
        }
        
        // Add all collected functions
        for function in functions_to_add {
            self.abi.add_function(function);
        }
    }

    /// Collect functions from router match patterns
    fn collect_functions_from_router(&self, lines: &[&str], router_line_index: usize, functions: &mut Vec<FunctionAbi>) {
        // Look for match patterns like: 0x01 => function_name(call.args)
        for i in router_line_index..lines.len() {
            let line = lines[i].trim();
            
            if line.starts_with("0x") && line.contains("=>") {
                println!("Processing line: {}", line);
                
                // Extract selector from current line
                let parts: Vec<&str> = line.split("=>").collect();
                if parts.len() != 2 {
                    continue;
                }
                
                let selector_str = parts[0].trim();
                if !selector_str.starts_with("0x") {
                    continue;
                }
                
                let selector = match u8::from_str_radix(&selector_str[2..], 16) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                
                // Look for function call on the next few lines
                let mut function_name = None;
                for j in (i + 1)..(i + 5).min(lines.len()) {
                    let next_line = lines[j].trim();
                    if next_line.starts_with('}') {
                        break;
                    }
                    
                    // Look for function call pattern: function_name(...)
                    if let Some(start) = next_line.find('(') {
                        let potential_name = next_line[..start].trim();
                        if !potential_name.is_empty() && 
                           !potential_name.contains(' ') && 
                           !potential_name.contains("::") &&  // Skip method calls like Result::with_u32
                           !potential_name.contains('.') {     // Skip method calls
                            function_name = Some(potential_name);
                            break;
                        }
                    }
                    
                    // Also look for patterns like "let b = balance_of(call.args)"
                    if next_line.contains("let") && next_line.contains('=') {
                        if let Some(equals_pos) = next_line.find('=') {
                            let after_equals = &next_line[equals_pos + 1..];
                            if let Some(start) = after_equals.find('(') {
                                let potential_name = after_equals[..start].trim();
                                if !potential_name.is_empty() && 
                                   !potential_name.contains(' ') && 
                                   !potential_name.contains("::") && 
                                   !potential_name.contains('.') {
                                    function_name = Some(potential_name);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                if let Some(name) = function_name {
                    println!("Found function: selector={}, name='{}'", selector, name);
                    // Try to find the function definition
                    if let Some(mut function) = self.find_function_definition(lines, name) {
                        function.selector = selector;
                        functions.push(function);
                    } else {
                        // If we can't find the function definition, create a basic one
                        println!("Creating basic function for: {}", name);
                        functions.push(FunctionAbi {
                            name: name.to_string(),
                            selector,
                            inputs: Vec::new(),
                            outputs: vec![ParamType::Uint(32)],
                        });
                    }
                } else {
                    println!("Failed to find function name for selector: {}", selector);
                }
            }
            
            // Stop when we hit the end of the match block
            if line == "}" {
                break;
            }
        }
    }

    /// Parse a selector pattern like "0x01 => function_name(call.args)" or "0x01 => { function_name(caller, call.args); }"
    fn parse_selector_pattern<'a>(&self, line: &'a str) -> Option<(u8, &'a str)> {
        let parts: Vec<&str> = line.split("=>").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let selector_str = parts[0].trim();
        let function_part = parts[1].trim();
        
        // Extract selector
        if !selector_str.starts_with("0x") {
            return None;
        }
        
        let selector = u8::from_str_radix(&selector_str[2..], 16).ok()?;
        
        // Extract function name - handle both patterns:
        // 1. "function_name(call.args)" 
        // 2. "{ function_name(caller, call.args); }"
        let function_name = if function_part.starts_with('{') {
            // Pattern 2: extract from inside braces
            let inner = function_part.trim_start_matches('{').trim_end_matches('}');
            // Look for function call pattern: function_name(...)
            if let Some(start) = inner.find('(') {
                inner[..start].trim()
            } else {
                return None;
            }
        } else {
            // Pattern 1: extract directly
            if let Some(start) = function_part.find('(') {
                function_part[..start].trim()
            } else {
                return None;
            }
        };
        
        // Skip empty function names
        if function_name.is_empty() {
            return None;
        }
        
        Some((selector, function_name))
    }

    /// Find function definition and create FunctionAbi
    fn find_function_definition(&self, lines: &[&str], function_name: &str) -> Option<FunctionAbi> {
        for line in lines {
            if line.contains(&format!("fn {}", function_name)) {
                // For now, create a basic function ABI
                // In a full implementation, you'd parse the function signature
                return Some(FunctionAbi {
                    name: function_name.to_string(),
                    selector: 0, // Will be set by the router parser
                    inputs: Vec::new(), // Would parse from function signature
                    outputs: vec![ParamType::Uint(32)], // Default output
                });
            }
        }
        None
    }

    /// Generate ABI from a source file
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<ContractAbi> {
        let source_code = fs::read_to_string(path)?;
        let mut generator = AbiGenerator::new(source_code);
        Ok(generator.generate())
    }

    /// Write ABI to a JSON file
    pub fn write_abi_to_file<P: AsRef<Path>>(abi: &ContractAbi, path: P) -> std::io::Result<()> {
        let json = abi.to_json();
        fs::write(path, json)
    }
}

/// Generate ABI for all example programs
pub fn generate_all_example_abis() -> std::io::Result<()> {
    let examples_dir = "crates/examples/src";
    let bin_dir = "crates/examples/bin";
    
    // Ensure bin directory exists
    fs::create_dir_all(bin_dir)?;
    
    let source_files = vec![
        "simple.rs",
        "storage.rs", 
        "erc20.rs",
        "multi_func.rs",
        "call_program.rs",
    ];
    
    for source_file in source_files {
        let source_path = format!("{}/{}", examples_dir, source_file);
        let abi_path = format!("{}/{}.abi.json", bin_dir, source_file.replace(".rs", ""));
        
        println!("Generating ABI for {}", source_file);
        
        match AbiGenerator::from_file(&source_path) {
            Ok(abi) => {
                AbiGenerator::write_abi_to_file(&abi, &abi_path)?;
                println!("  ✓ Generated {}", abi_path);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to generate ABI for {}: {}", source_file, e);
            }
        }
    }
    
    Ok(())
}
