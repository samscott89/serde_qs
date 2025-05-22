use std::{borrow::Cow, str::Utf8Error};

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
pub fn decode(input: &[u8], strict: bool) -> Result<Cow<'_, str>, Utf8Error> {
    let mut bytes_iter = input.iter().enumerate();

    let mut decoded = None;
    let mut last_segment = 0;

    while let Some((idx, &b)) = bytes_iter.next() {
        if b == b'+' {
            let decoded = decoded.get_or_insert_with(|| Vec::with_capacity(input.len()));
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

            let decoded = decoded.get_or_insert_with(|| Vec::with_capacity(input.len()));
            decoded.extend_from_slice(&input[last_segment..idx]);

            let decoded_byte = h as u8 * 0x10 + l as u8;
            decoded.push(decoded_byte);
            last_segment = idx + 3;
        }
    }

    if let Some(mut decoded) = decoded {
        decoded.extend_from_slice(&input[last_segment..]);
        if strict {
            String::from_utf8(decoded).map_err(|e| e.utf8_error())
        } else {
            Ok(
                // TODO(Sam): replace this with `String::from_utf8_lossy_owned`
                // when it stabilizes
                if let Cow::Owned(string) = String::from_utf8_lossy(&decoded) {
                    string
                } else {
                    // SAFETY: `String::from_utf8_lossy`'s contract ensures that if
                    // it returns a `Cow::Borrowed`, it is a valid UTF-8 string.
                    // Otherwise, it returns a new allocation of an owned `String`, with
                    // replacement characters for invalid sequences, which is returned
                    // above.
                    unsafe { String::from_utf8_unchecked(decoded) }
                },
            )
        }
        .map(Cow::Owned)
    } else {
        // No percent-encoded characters found, just convert to UTF-8
        std::str::from_utf8(input).map(Cow::Borrowed)
    }
}
