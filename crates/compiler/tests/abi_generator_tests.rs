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
