#[derive(Debug, Clone)]
pub struct EventAbi {
    pub name: String,
    pub inputs: Vec<EventParam>,
}

#[derive(Debug, Clone)]
pub struct EventParam {
    pub name: String,
    pub kind: ParamType,
    pub indexed: bool,
}

impl EventAbi {
    /// Returns the event ID: first 32 bytes of the event name, padded with zeros.
    pub fn id(&self) -> [u8; 32] {
        let name_bytes = self.name.as_bytes();
        let mut id = [0u8; 32];
        let len = name_bytes.len().min(32);
        id[..len].copy_from_slice(&name_bytes[..len]);
        id
    }
}

#[derive(Debug, Clone)]
pub struct FunctionAbi {
    pub name: String,
    pub selector: u8,
    pub inputs: Vec<FunctionParam>,
    pub outputs: Vec<ParamType>,
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub kind: ParamType,
}

#[derive(Debug, Clone)]
pub enum ParamType {
    Address,
    Uint(usize), // bits
    Bool,
    Bytes,
    String,
    // Extend as needed
}

impl ParamType {
    /// Convert to JSON-compatible string representation
    pub fn to_json_string(&self) -> String {
        match self {
            ParamType::Address => "address".to_string(),
            ParamType::Uint(bits) => format!("uint{}", bits),
            ParamType::Bool => "bool".to_string(),
            ParamType::Bytes => "bytes".to_string(),
            ParamType::String => "string".to_string(),
        }
    }
}

/// Complete ABI for a smart contract
#[derive(Debug, Clone)]
pub struct ContractAbi {
    pub version: String,
    pub functions: Vec<FunctionAbi>,
    pub events: Vec<EventAbi>,
}

impl ContractAbi {
    /// Create a new empty ABI
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            functions: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Add a function to the ABI
    pub fn add_function(&mut self, function: FunctionAbi) {
        self.functions.push(function);
    }

    /// Add an event to the ABI
    pub fn add_event(&mut self, event: EventAbi) {
        self.events.push(event);
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"version\": \"{}\",\n", self.version));
        
        // Functions
        json.push_str("  \"functions\": [\n");
        for (i, func) in self.functions.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", func.name));
            json.push_str(&format!("      \"selector\": {},\n", func.selector));
            
            // Inputs
            json.push_str("      \"inputs\": [\n");
            for (j, input) in func.inputs.iter().enumerate() {
                json.push_str("        {\n");
                json.push_str(&format!("          \"name\": \"{}\",\n", input.name));
                json.push_str(&format!("          \"type\": \"{}\"\n", input.kind.to_json_string()));
                if j < func.inputs.len() - 1 {
                    json.push_str("        },\n");
                } else {
                    json.push_str("        }\n");
                }
            }
            json.push_str("      ],\n");
            
            // Outputs
            json.push_str("      \"outputs\": [\n");
            for (j, output) in func.outputs.iter().enumerate() {
                json.push_str("        {\n");
                json.push_str(&format!("          \"type\": \"{}\"\n", output.to_json_string()));
                if j < func.outputs.len() - 1 {
                    json.push_str("        },\n");
                } else {
                    json.push_str("        }\n");
                }
            }
            json.push_str("      ]\n");
            
            if i < self.functions.len() - 1 {
                json.push_str("    },\n");
            } else {
                json.push_str("    }\n");
            }
        }
        json.push_str("  ],\n");
        
        // Events
        json.push_str("  \"events\": [\n");
        for (i, event) in self.events.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", event.name));
            json.push_str("      \"inputs\": [\n");
            for (j, input) in event.inputs.iter().enumerate() {
                json.push_str("        {\n");
                json.push_str(&format!("          \"name\": \"{}\",\n", input.name));
                json.push_str(&format!("          \"type\": \"{}\",\n", input.kind.to_json_string()));
                json.push_str(&format!("          \"indexed\": {}\n", input.indexed));
                if j < event.inputs.len() - 1 {
                    json.push_str("        },\n");
                } else {
                    json.push_str("        }\n");
                }
            }
            json.push_str("      ]\n");
            
            if i < self.events.len() - 1 {
                json.push_str("    },\n");
            } else {
                json.push_str("    }\n");
            }
        }
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        json
    }
}
