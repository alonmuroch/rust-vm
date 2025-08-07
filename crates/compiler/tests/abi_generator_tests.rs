use compiler::{AbiGenerator, ContractAbi, EventAbi, EventParam, FunctionAbi, FunctionParam, ParamType};

#[test]
fn test_abi_generation() {
    // Test with a simple program that has events
    let source_code = r#"
        event!(Minted {
            caller => Address,
            amount => u32,
        });
        
        event!(Transfer {
            from => Address,
            to => Address,
            value => u32,
        });
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    assert_eq!(abi.events.len(), 2);
    assert_eq!(abi.events[0].name, "Minted");
    assert_eq!(abi.events[1].name, "Transfer");
    
    println!("Generated ABI: {}", abi.to_json());
}

#[test]
fn test_function_extraction() {
    // Test with a program that has functions with selectors
    let source_code = r#"
        unsafe fn main_entry(program: Address, caller: Address, data: &[u8]) -> Result {   
            route(data, program, caller, 
                 |to, from, call| match call.selector {
                0x01 => {
                    init(caller, call.args);
                    Result::new(true, 0)
                },
                0x02 => {
                    transfer(caller, call.args);
                    Result::new(true, 0)
                },
                0x05 => {
                    let b = balance_of(call.args);
                    Result::with_u32(b)
                },
                _ => vm_panic(b"unknown selector"),
            })
        }

        fn init(caller: Address, args: &[u8]) {
            // function body
        }

        fn transfer(caller: Address, args: &[u8]) {
            // function body
        }

        fn balance_of(args: &[u8]) -> u32 {
            0
        }
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    assert_eq!(abi.functions.len(), 3);
    
    // Check that all expected functions are found
    let function_names: Vec<&str> = abi.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(function_names.contains(&"init"));
    assert!(function_names.contains(&"transfer"));
    assert!(function_names.contains(&"balance_of"));
    
    // Check selectors
    let init_func = abi.functions.iter().find(|f| f.name == "init").unwrap();
    let transfer_func = abi.functions.iter().find(|f| f.name == "transfer").unwrap();
    let balance_func = abi.functions.iter().find(|f| f.name == "balance_of").unwrap();
    
    assert_eq!(init_func.selector, 1);
    assert_eq!(transfer_func.selector, 2);
    assert_eq!(balance_func.selector, 5);
}

#[test]
fn test_multi_line_event_parsing() {
    // Test with multi-line event definition
    let source_code = r#"
        event!(ComplexEvent {
            user => Address,
            amount => u32,
            timestamp => u64,
            metadata => String,
        });
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    assert_eq!(abi.events.len(), 1);
    let event = &abi.events[0];
    assert_eq!(event.name, "ComplexEvent");
    assert_eq!(event.inputs.len(), 4);
    
    // Check parameter types
    assert_eq!(event.inputs[0].name, "user");
    assert!(matches!(event.inputs[0].kind, ParamType::Address));
    assert_eq!(event.inputs[1].name, "amount");
    assert!(matches!(event.inputs[1].kind, ParamType::Uint(32)));
    assert_eq!(event.inputs[2].name, "timestamp");
    assert!(matches!(event.inputs[2].kind, ParamType::Uint(64)));
    assert_eq!(event.inputs[3].name, "metadata");
    assert!(matches!(event.inputs[3].kind, ParamType::String));
}

#[test]
fn test_empty_program() {
    // Test with a program that has no events or functions
    let source_code = r#"
        fn main() {
            // Just a simple function
        }
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    assert_eq!(abi.events.len(), 0);
    assert_eq!(abi.functions.len(), 0);
}

#[test]
fn test_contract_abi_json_generation() {
    // Test complete ABI JSON generation
    let mut abi = ContractAbi::new();
    
    // Add an event
    abi.add_event(EventAbi {
        name: "TestEvent".to_string(),
        inputs: vec![
            EventParam {
                name: "param1".to_string(),
                kind: ParamType::Address,
                indexed: false,
            },
            EventParam {
                name: "param2".to_string(),
                kind: ParamType::Uint(32),
                indexed: true,
            },
        ],
    });
    
    // Add a function
    abi.add_function(FunctionAbi {
        name: "testFunction".to_string(),
        selector: 1,
        inputs: vec![
            FunctionParam {
                name: "input1".to_string(),
                kind: ParamType::Address,
            },
        ],
        outputs: vec![ParamType::Uint(32)],
    });
    
    let json = abi.to_json();
    
    // Verify JSON contains expected content
    assert!(json.contains("\"name\": \"TestEvent\""));
    assert!(json.contains("\"name\": \"testFunction\""));
    assert!(json.contains("\"selector\": 1"));
    assert!(json.contains("\"type\": \"address\""));
    assert!(json.contains("\"type\": \"uint32\""));
    assert!(json.contains("\"indexed\": true"));
    assert!(json.contains("\"indexed\": false"));
}

#[test]
fn test_real_erc20_like_program() {
    // Test with a realistic ERC20-like program
    let source_code = r#"
        event!(Transfer {
            from => Address,
            to => Address,
            value => u32,
        });

        event!(Minted {
            caller => Address,
            amount => u32,
        });

        unsafe fn main_entry(program: Address, caller: Address, data: &[u8]) -> Result {   
            route(data, program, caller, 
                 |to, from, call| match call.selector {
                0x01 => {
                    init(caller, call.args);
                    Result::new(true, 0)
                },
                0x02 => {
                    transfer(caller, call.args);
                    Result::new(true, 0)
                },
                0x05 => {
                    let b = balance_of(call.args);
                    Result::with_u32(b)
                },
                _ => vm_panic(b"unknown selector"),
            })
        }

        fn init(caller: Address, args: &[u8]) {
            // Initialize contract
        }

        fn transfer(caller: Address, args: &[u8]) {
            // Transfer tokens
            fire_event!(Transfer::new(caller, caller, 0));
        }

        fn balance_of(args: &[u8]) -> u32 {
            0
        }
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    // Check events
    assert_eq!(abi.events.len(), 2);
    let event_names: Vec<&str> = abi.events.iter().map(|e| e.name.as_str()).collect();
    assert!(event_names.contains(&"Transfer"));
    assert!(event_names.contains(&"Minted"));
    
    // Check functions
    assert_eq!(abi.functions.len(), 3);
    let function_names: Vec<&str> = abi.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(function_names.contains(&"init"));
    assert!(function_names.contains(&"transfer"));
    assert!(function_names.contains(&"balance_of"));
    
    // Check selectors
    let init_func = abi.functions.iter().find(|f| f.name == "init").unwrap();
    let transfer_func = abi.functions.iter().find(|f| f.name == "transfer").unwrap();
    let balance_func = abi.functions.iter().find(|f| f.name == "balance_of").unwrap();
    
    assert_eq!(init_func.selector, 1);
    assert_eq!(transfer_func.selector, 2);
    assert_eq!(balance_func.selector, 5);
}

#[test]
fn test_param_type_json_conversion() {
    // Test that ParamType converts to JSON correctly
    assert_eq!(ParamType::Address.to_json_string(), "address");
    assert_eq!(ParamType::Uint(32).to_json_string(), "uint32");
    assert_eq!(ParamType::Uint(64).to_json_string(), "uint64");
    assert_eq!(ParamType::Uint(8).to_json_string(), "uint8");
    assert_eq!(ParamType::Bool.to_json_string(), "bool");
    assert_eq!(ParamType::String.to_json_string(), "string");
    assert_eq!(ParamType::Bytes.to_json_string(), "bytes");
}

#[test]
fn test_function_input_extraction() {
    // Test with functions that have proper input parameters
    let source_code = r#"
        fn init(caller: Address, args: &[u8]) {
            // function body
        }

        fn transfer(caller: Address, args: &[u8]) {
            // function body
        }

        fn balance_of(args: &[u8]) -> u32 {
            0
        }

        unsafe fn main_entry(program: Address, caller: Address, data: &[u8]) -> Result {   
            route(data, program, caller, 
                 |to, from, call| match call.selector {
                0x01 => {
                    init(caller, call.args);
                    Result::new(true, 0)
                },
                0x02 => {
                    transfer(caller, call.args);
                    Result::new(true, 0)
                },
                0x05 => {
                    let b = balance_of(call.args);
                    Result::with_u32(b)
                },
                _ => vm_panic(b"unknown selector"),
            })
        }
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    // Check that all functions have the correct inputs
    assert_eq!(abi.functions.len(), 3);
    
    // Check init function
    let init_func = abi.functions.iter().find(|f| f.name == "init").unwrap();
    assert_eq!(init_func.inputs.len(), 2);
    assert_eq!(init_func.inputs[0].name, "caller");
    assert!(matches!(init_func.inputs[0].kind, ParamType::Address));
    assert_eq!(init_func.inputs[1].name, "args");
    assert!(matches!(init_func.inputs[1].kind, ParamType::Bytes));
    
    // Check transfer function
    let transfer_func = abi.functions.iter().find(|f| f.name == "transfer").unwrap();
    assert_eq!(transfer_func.inputs.len(), 2);
    assert_eq!(transfer_func.inputs[0].name, "caller");
    assert!(matches!(transfer_func.inputs[0].kind, ParamType::Address));
    assert_eq!(transfer_func.inputs[1].name, "args");
    assert!(matches!(transfer_func.inputs[1].kind, ParamType::Bytes));
    
    // Check balance_of function
    let balance_func = abi.functions.iter().find(|f| f.name == "balance_of").unwrap();
    assert_eq!(balance_func.inputs.len(), 1);
    assert_eq!(balance_func.inputs[0].name, "args");
    assert!(matches!(balance_func.inputs[0].kind, ParamType::Bytes));
    assert_eq!(balance_func.outputs.len(), 1);
    assert!(matches!(balance_func.outputs[0], ParamType::Uint(32)));
}

#[test]
fn test_function_signature_parsing() {
    // Test parsing of various function signature patterns
    let source_code = r#"
        fn simple_function() {
            // no parameters
        }

        fn single_param(param: u32) {
            // single parameter
        }

        fn multiple_params(a: Address, b: u64, c: bool) {
            // multiple parameters
        }

        fn with_return_type(input: String) -> u32 {
            0
        }

        fn complex_signature(
            caller: Address,
            args: &[u8],
            flag: bool
        ) -> u32 {
            0
        }
    "#;
    
    let mut generator = AbiGenerator::new(source_code.to_string());
    let abi = generator.generate();
    
    // Since these functions aren't called in the router, they won't be included
    // But we can test the signature parsing directly
    let lines: Vec<&str> = source_code.lines().collect();
    
    // Test simple_function
    if let Some(signature) = generator.extract_function_signature(&lines, 1) {
        let func = generator.parse_function_signature("simple_function", signature);
        assert_eq!(func.inputs.len(), 0);
    }
    
    // Test single_param
    if let Some(signature) = generator.extract_function_signature(&lines, 5) {
        let func = generator.parse_function_signature("single_param", signature);
        assert_eq!(func.inputs.len(), 1);
        assert_eq!(func.inputs[0].name, "param");
        assert!(matches!(func.inputs[0].kind, ParamType::Uint(32)));
    }
    
    // Test multiple_params
    if let Some(signature) = generator.extract_function_signature(&lines, 9) {
        let func = generator.parse_function_signature("multiple_params", signature);
        assert_eq!(func.inputs.len(), 3);
        assert_eq!(func.inputs[0].name, "a");
        assert!(matches!(func.inputs[0].kind, ParamType::Address));
        assert_eq!(func.inputs[1].name, "b");
        assert!(matches!(func.inputs[1].kind, ParamType::Uint(64)));
        assert_eq!(func.inputs[2].name, "c");
        assert!(matches!(func.inputs[2].kind, ParamType::Bool));
    }
    
    // Test with_return_type
    if let Some(signature) = generator.extract_function_signature(&lines, 13) {
        let func = generator.parse_function_signature("with_return_type", signature);
        assert_eq!(func.inputs.len(), 1);
        assert_eq!(func.inputs[0].name, "input");
        assert!(matches!(func.inputs[0].kind, ParamType::String));
        assert_eq!(func.outputs.len(), 1);
        assert!(matches!(func.outputs[0], ParamType::Uint(32)));
    }
}

#[test]
fn test_param_type_parsing() {
    // Test that ParamType parsing works correctly
    let generator = AbiGenerator::new(String::new());
    
    assert_eq!(generator.parse_param_type_from_str("Address"), Some(ParamType::Address));
    assert_eq!(generator.parse_param_type_from_str("u32"), Some(ParamType::Uint(32)));
    assert_eq!(generator.parse_param_type_from_str("u64"), Some(ParamType::Uint(64)));
    assert_eq!(generator.parse_param_type_from_str("u8"), Some(ParamType::Uint(8)));
    assert_eq!(generator.parse_param_type_from_str("bool"), Some(ParamType::Bool));
    assert_eq!(generator.parse_param_type_from_str("String"), Some(ParamType::String));
    assert_eq!(generator.parse_param_type_from_str("&[u8]"), Some(ParamType::Bytes));
    assert_eq!(generator.parse_param_type_from_str("[u8]"), Some(ParamType::Bytes));
    assert_eq!(generator.parse_param_type_from_str("unknown"), None);
}
