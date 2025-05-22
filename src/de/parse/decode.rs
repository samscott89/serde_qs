use std::borrow::Cow;

#[inline(always)]
fn char_to_digit(c: u8) -> Option<u32> {
    char::from(c).to_digit(16)
}

/// Decodes the input string, applying the following:
/// - Replaces `+` with a space
/// - Decodes percent-encoded characters
/// - Converts the result to a UTF-8 string
///
/// If `strict` is true, it will return an error if the input contains invalid UTF-8.
///
/// This code is adapted from `rust-url` which contains each of the following
/// in slightly separate functions.
pub fn decode(input: &[u8]) -> Cow<'_, [u8]> {
    if !input.iter().any(|&b| b == b'+' || b == b'%') {
        // No percent-encoded characters found, just convert to UTF-8
        return Cow::Borrowed(input);
    }

    let mut bytes_iter = input.iter().enumerate();

    let mut decoded = Vec::with_capacity(input.len());
    let mut last_segment = 0;

    while let Some((idx, &b)) = bytes_iter.next() {
        if b == b'+' {
            decoded.extend_from_slice(&input[last_segment..idx]);
            // push space
            decoded.push(b' ');
            last_segment = idx + 1;
        } else if b == b'%' {
            // Decode percent-encoded characters

            // first attempt to decode the next two bytes
            // if this fails, we'll skip over the invalid percent-encoded character
            let Some(h) = bytes_iter.next().and_then(|(_, b)| char_to_digit(*b)) else {
                continue;
            };
            let Some(l) = bytes_iter.next().and_then(|(_, b)| char_to_digit(*b)) else {
                continue;
            };

            decoded.extend_from_slice(&input[last_segment..idx]);

            let decoded_byte = h as u8 * 0x10 + l as u8;
            decoded.push(decoded_byte);
            last_segment = idx + 3;
        }
    }

    decoded.extend_from_slice(&input[last_segment..]);
    Cow::Owned(decoded)
}
