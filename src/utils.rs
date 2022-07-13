use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use std::borrow::Cow;

pub const QS_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b' ')
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

pub fn replace_space(input: &str) -> Cow<str> {
    match input.as_bytes().iter().position(|&b| b == b' ') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.as_bytes().to_owned();
            replaced[first_position] = b'+';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b' ' {
                    *byte = b'+';
                }
            }
            Cow::Owned(String::from_utf8(replaced).expect("replacing ' ' with '+' cannot panic"))
        }
    }
}
