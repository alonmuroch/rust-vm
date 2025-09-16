use program::router::{decode_calls, route, FuncCall};
use types::{Result, Address};

#[test]
fn test_decode_single_call() {
    println!("=== Testing Router: Decode Single Call ===");
    let input = [0x01, 0x08, 100, 0, 0, 0, 42, 0, 0, 0];
    println!("Input: selector=0x01, arg_len=0x08, args=[100, 0, 0, 0, 42, 0, 0, 0]");

    let mut buf = [FuncCall { selector: 0, args: &[] }; 1];

    let count = decode_calls(&input, &mut buf);
    println!("Decoded {} call(s)", count);

    assert_eq!(count, 1);
    assert_eq!(buf[0].selector, 0x01);
    assert_eq!(buf[0].args, &[100, 0, 0, 0, 42, 0, 0, 0]);

    println!("✓ Single call decoded successfully");
}

#[test]
fn test_decode_multiple_calls() {
    println!("=== Testing Router: Decode Multiple Calls ===");
    let input = [
        0x01, 0x02, 1, 2,
        0x02, 0x03, 3, 4, 5,
    ];
    println!("Input: Two calls - Call1: [0x01, 0x02, 1, 2], Call2: [0x02, 0x03, 3, 4, 5]");

    let mut buf = [FuncCall { selector: 0, args: &[] }; 2];

    let count = decode_calls(&input, &mut buf);
    println!("Decoded {} call(s)", count);

    assert_eq!(count, 2);
    assert_eq!(buf[0].selector, 0x01);
    assert_eq!(buf[0].args, &[1, 2]);
    println!("  Call 1: selector=0x{:02x}, args={:?}", buf[0].selector, buf[0].args);

    assert_eq!(buf[1].selector, 0x02);
    assert_eq!(buf[1].args, &[3, 4, 5]);
    println!("  Call 2: selector=0x{:02x}, args={:?}", buf[1].selector, buf[1].args);

    println!("✓ Multiple calls decoded successfully");
}

#[test]
fn test_route_with_two_calls() {
    println!("=== Testing Router: Route with Two Calls ===");
    let input = [
        0x10, 0x01, 42,
        0x20, 0x02, 1, 2,
    ];
    println!("Input: Two calls - Call1: [0x10, 0x01, 42], Call2: [0x20, 0x02, 1, 2]");

    let to = Address([0u8; 20]);
    let from = Address([0u8; 20]);

    println!("Routing calls through handler...");
    let result = route(&input, to, from, |_to, _from, call| {
                    match call.selector {
                0x10 => {
                    println!("  Processing call 0x10 with args: {:?}", call.args);
                    assert_eq!(call.args, &[42]);
                    Result::new(true, 10)
                },
                0x20 => {
                    println!("  Processing call 0x20 with args: {:?}", call.args);
                    assert_eq!(call.args, &[1, 2]);
                    Result::new(false, 20)
                },
                _ => Result::new(false, 999),
            }
    });

    let error_code = result.error_code;
    println!("Final result: success={}, error_code={}", result.success, error_code);
    assert_eq!(result, Result::new(false, 20));
    println!("✓ Route with two calls completed successfully");
}

#[test]
#[should_panic(expected = "vm_panic: decode: incomplete header")]
fn test_decode_incomplete_header_panics() {
    println!("=== Testing Router: Decode with Incomplete Header ===");
    let input = [0x01];
    println!("Input: [0x01] - Missing arg_len byte");
    println!("Expected: Should panic with 'decode: incomplete header'");

    let mut buf = [FuncCall { selector: 0, args: &[] }; 1];
    decode_calls(&input, &mut buf);
}

#[test]
#[should_panic(expected = "vm_panic: decode: args too short")]
fn test_decode_args_too_short_panics() {
    println!("=== Testing Router: Decode with Insufficient Arguments ===");
    let input = [0x01, 0x04, 1, 2]; // 4 bytes expected, only 2 given
    println!("Input: [0x01, 0x04, 1, 2] - Claims 4 arg bytes but only has 2");
    println!("Expected: Should panic with 'decode: args too short'");

    let mut buf = [FuncCall { selector: 0, args: &[] }; 1];
    decode_calls(&input, &mut buf);
}
