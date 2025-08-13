use std::fs;
use std::path::Path;
use crate::abi::{ContractAbi, FunctionAbi, ParamType};

/// ABI Code Generator that creates Rust client code from ABI definitions
pub struct AbiCodeGenerator {
    abi: ContractAbi,
    contract_name: String,
}

impl AbiCodeGenerator {
    /// Create a new ABI code generator
    pub fn new(abi: ContractAbi, contract_name: String) -> Self {
        Self {
            abi,
            contract_name,
        }
    }

    /// Generate Rust client code from the ABI
    pub fn generate_rust_code(&self) -> String {
        let mut code = String::new();
        
        // Add header - no attributes since this will be included
        code.push_str("// Auto-generated ABI client code\n");
        code.push_str("// DO NOT EDIT - Generated from ABI\n\n");
        
        // Don't add imports - assume they're in the parent file
        code.push_str("// Note: This code assumes the following imports in the parent file:\n");
        code.push_str("// use program::types::address::Address;\n");
        code.push_str("// use program::types::result::Result;\n");
        code.push_str("// use program::call::call;\n\n");
        
        // Generate contract struct
        code.push_str(&format!("/// Client for interacting with {} contract\n", self.contract_name));
        code.push_str(&format!("pub struct {} {{\n", self.contract_name));
        code.push_str("    pub address: Address,\n");
        code.push_str("}\n\n");
        
        // Generate implementation
        code.push_str(&format!("impl {} {{\n", self.contract_name));
        
        // Constructor
        code.push_str("    /// Create a new contract client\n");
        code.push_str("    pub fn new(address: Address) -> Self {\n");
        code.push_str("        Self { address }\n");
        code.push_str("    }\n\n");
        
        // If there are no routed functions, generate a call_main method
        if self.abi.functions.is_empty() {
            code.push_str("    /// Call the main entry point directly (no routing)\n");
            code.push_str("    pub fn call_main(\n");
            code.push_str("        &self,\n");
            code.push_str("        caller: &Address,\n");
            code.push_str("        data: &[u8],\n");
            code.push_str("    ) -> Option<Result> {\n");
            code.push_str("        // Direct call without router encoding\n");
            code.push_str("        call(caller, &self.address, data)\n");
            code.push_str("    }\n\n");
        } else {
            // Generate methods for each function
            for function in &self.abi.functions {
                code.push_str(&self.generate_function_method(function));
                code.push_str("\n");
            }
        }
        
        code.push_str("}\n");
        
        code
    }
    
    /// Generate a method for a single function
    fn generate_function_method(&self, function: &FunctionAbi) -> String {
        let mut method = String::new();
        
        // Add documentation
        method.push_str(&format!("    /// Call the {} function\n", function.name));
        
        // Generate method signature
        method.push_str(&format!("    pub fn {}(\n", function.name));
        method.push_str("        &self,\n");
        method.push_str("        caller: &Address,\n");
        
        // Add function parameters
        for input in &function.inputs {
            let rust_type = self.param_type_to_rust(&input.kind);
            method.push_str(&format!("        {}: {},\n", input.name, rust_type));
        }
        
        method.push_str("    ) -> Option<Result> {\n");
        
        // Generate method body
        if function.selector > 0 {
            // Use manual router encoding for functions with selectors
            method.push_str("        // Encode router call manually\n");
            
            // For simple byte slice inputs
            if function.inputs.len() == 1 && matches!(function.inputs[0].kind, ParamType::Bytes) {
                method.push_str(&format!("        let arg_len = {}.len() as u8;\n", function.inputs[0].name));
                method.push_str("        let mut encoded = [0u8; 256]; // Fixed buffer\n");
                method.push_str(&format!("        encoded[0] = 0x{:02x}; // selector\n", function.selector));
                method.push_str("        encoded[1] = arg_len; // arg length\n");
                method.push_str(&format!("        encoded[2..2 + arg_len as usize].copy_from_slice({});\n", function.inputs[0].name));
                method.push_str("        let data = &encoded[..2 + arg_len as usize];\n");
            } else {
                // For other types, we'd need custom encoding
                method.push_str(&format!("        let data = &[0x{:02x}, 0]; // selector + empty args\n", function.selector));
            }
            
            method.push_str("        \n");
            method.push_str("        // Make the call\n");
            method.push_str("        call(caller, &self.address, data)\n");
        } else {
            // Direct call without router encoding
            if function.inputs.len() == 1 && matches!(function.inputs[0].kind, ParamType::Bytes) {
                method.push_str("        // Make the call with raw data\n");
                method.push_str(&format!("        call(caller, &self.address, {})\n", function.inputs[0].name));
            } else {
                method.push_str("        // Make the call\n");
                method.push_str("        call(caller, &self.address, &[])\n");
            }
        }
        
        method.push_str("    }\n");
        
        method
    }
    
    /// Convert ParamType to Rust type string
    fn param_type_to_rust(&self, param_type: &ParamType) -> String {
        match param_type {
            ParamType::Address => "Address".to_string(),
            ParamType::Uint(8) => "u8".to_string(),
            ParamType::Uint(16) => "u16".to_string(),
            ParamType::Uint(32) => "u32".to_string(),
            ParamType::Uint(64) => "u64".to_string(),
            ParamType::Uint(128) => "u128".to_string(),
            ParamType::Uint(256) => "[u8; 32]".to_string(), // 256-bit as byte array
            ParamType::Bool => "bool".to_string(),
            ParamType::String => "&str".to_string(),
            ParamType::Bytes => "&[u8]".to_string(),
            ParamType::Result => "Result".to_string(),
            _ => "Vec<u8>".to_string(),
        }
    }
    
    /// Generate code to encode an argument
    fn generate_argument_encoding(&self, name: &str, param_type: &ParamType) -> String {
        match param_type {
            ParamType::Address => {
                format!("        args.extend({}.0.to_vec());\n", name)
            }
            ParamType::Uint(8) => {
                format!("        args.push({});\n", name)
            }
            ParamType::Uint(16) => {
                format!("        args.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(32) => {
                format!("        args.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(64) => {
                format!("        args.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(128) => {
                format!("        args.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Bool => {
                format!("        args.push(if {} {{ 1 }} else {{ 0 }});\n", name)
            }
            ParamType::String => {
                format!("        args.extend({}.as_bytes());\n", name)
            }
            ParamType::Bytes => {
                format!("        args.extend({});\n", name)
            }
            _ => {
                format!("        // TODO: Encode {}\n", name)
            }
        }
    }
    
    /// Generate code to encode an argument for direct calls
    fn generate_argument_encoding_direct(&self, name: &str, param_type: &ParamType) -> String {
        match param_type {
            ParamType::Address => {
                format!("        data.extend({}.0.to_vec());\n", name)
            }
            ParamType::Uint(8) => {
                format!("        data.push({});\n", name)
            }
            ParamType::Uint(16) => {
                format!("        data.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(32) => {
                format!("        data.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(64) => {
                format!("        data.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Uint(128) => {
                format!("        data.extend({}.to_le_bytes());\n", name)
            }
            ParamType::Bool => {
                format!("        data.push(if {} {{ 1 }} else {{ 0 }});\n", name)
            }
            ParamType::String => {
                format!("        data.extend({}.as_bytes());\n", name)
            }
            ParamType::Bytes => {
                format!("        data.extend({});\n", name)
            }
            _ => {
                format!("        // TODO: Encode {}\n", name)
            }
        }
    }
    
    /// Generate Rust client code from an ABI file
    pub fn from_abi_file<P: AsRef<Path>>(abi_path: P, contract_name: String) -> std::io::Result<String> {
        let abi_json = fs::read_to_string(abi_path)?;
        let abi = ContractAbi::from_json(&abi_json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let generator = AbiCodeGenerator::new(abi, contract_name);
        Ok(generator.generate_rust_code())
    }
    
    /// Write generated Rust code to a file
    pub fn write_to_file<P: AsRef<Path>>(code: &str, path: P) -> std::io::Result<()> {
        fs::write(path, code)
    }
}

/// Generate client code for all example ABIs
pub fn generate_all_client_code() -> std::io::Result<()> {
    let bin_dir = "crates/examples/bin";
    let generated_dir = "crates/examples/src/generated";
    
    // Ensure generated directory exists
    fs::create_dir_all(generated_dir)?;
    
    let abi_files = vec![
        ("simple.abi.json", "SimpleContract"),
        ("erc20.abi.json", "ERC20Contract"),
        ("multi_func.abi.json", "MultiFuncContract"),
    ];
    
    for (abi_file, contract_name) in abi_files {
        let abi_path = format!("{}/{}", bin_dir, abi_file);
        let output_path = format!("{}/{}.rs", generated_dir, abi_file.replace(".abi.json", "_client"));
        
        println!("Generating client code for {}", abi_file);
        
        match AbiCodeGenerator::from_abi_file(&abi_path, contract_name.to_string()) {
            Ok(code) => {
                AbiCodeGenerator::write_to_file(&code, &output_path)?;
                println!("  ✓ Generated {}", output_path);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to generate client code for {}: {}", abi_file, e);
            }
        }
    }
    
    Ok(())
}