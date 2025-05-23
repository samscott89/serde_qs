use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use std::borrow::Cow;

pub const QS_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b' ')
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

pub fn encode(b: &[u8]) -> Vec<u8> {
    percent_encoding::percent_encode(b, QS_ENCODE_SET)
        .flat_map(|b| {
            b.as_bytes()
                .iter()
                .map(|b| if *b == b' ' { b'+' } else { *b })
        })
        .collect()
}
