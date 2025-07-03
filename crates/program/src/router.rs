#[derive(Debug, Clone, Copy)]
pub struct FuncCall<'a> {
    pub selector: u8,
    pub args: &'a [u8],
}

use crate::{Result, vm_panic};

pub fn decode_calls<'a>(mut input: &'a [u8], buffer: &mut [FuncCall<'a>]) -> usize {
    let mut count = 0;

    while !input.is_empty() {
        if input.len() < 2 {
            vm_panic(b"decode: incomplete header");
        }

        let selector = input[0];
        let arg_len = input[1] as usize;

        if input.len() < 2 + arg_len {
            vm_panic(b"decode: args too short");
        }

        if count >= buffer.len() {
            vm_panic(b"decode: too many calls");
        }

        let args = &input[2..2 + arg_len];

        buffer[count] = FuncCall {
            selector,
            args,
        };

        count += 1;
        input = &input[2 + arg_len..];
    }

    count
}

/// Generic router that takes the input buffer and a closure to dispatch each call
pub fn route<'a>(
    input: &'a [u8],
    max_calls: usize,
    mut handler: impl FnMut(&FuncCall<'a>) -> Result,
) -> Result {
    let mut buf: [Option<FuncCall<'a>>; 8] = [None, None, None, None, None, None, None, None];
    let mut input = input;
    let mut count = 0;

    while !input.is_empty() && count < max_calls {
        if input.len() < 2 {
            vm_panic(b"router: bad header");
        }

        let selector = input[0];
        let arg_len = input[1] as usize;

        if input.len() < 2 + arg_len {
            vm_panic(b"router: bad arg len");
        }

        let args = &input[2..2 + arg_len];

        buf[count] = Some(FuncCall { selector, args });
        count += 1;
        input = &input[2 + arg_len..];
    }

    let mut last_result = Result {
        success: true,
        error_code: 0,
    };

    for i in 0..count {
        if let Some(call) = &buf[i] {
            last_result = handler(call);
        }
    }

    last_result
}