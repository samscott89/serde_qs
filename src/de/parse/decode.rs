use std::borrow::Cow;

#[inline(always)]
fn char_to_digit(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' {
        Some(c - b'0')
    } else {
        None
    }
}

/// Decodes the input string, applying the following:
/// - Replaces `+` with a space
/// - Decodes percent-encoded characters
#[inline(always)]
pub fn decode(input: &[u8]) -> Cow<'_, [u8]> {
    if input.is_empty() {
        return Cow::Borrowed(input);
    }

    if !input.iter().any(|&b| b == b'+' || b == b'%') {
        // No percent-encoded characters found, just convert to UTF-8
        return Cow::Borrowed(input);
    }

    let mut bytes_iter = input.iter().enumerate();

    // `decoded` is guaranteed to be large enough to hold the decoded string since
    // decoding a percent-encoded string will always result in a string
    // that is at most the same size as the original string.
    let mut decoded = Vec::with_capacity(input.len());
    let mut last_segment = 0;

    while let Some((idx, &b)) = bytes_iter.next() {
        if b == b'+' {
            extend_no_alloc(&mut decoded, &input[last_segment..idx]);
            push_no_alloc(&mut decoded, b' ');
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

            extend_no_alloc(&mut decoded, &input[last_segment..idx]);

            let decoded_byte = h * 0x10 + l;
            push_no_alloc(&mut decoded, decoded_byte);
            last_segment = idx + 3;
        }
    }

    extend_no_alloc(&mut decoded, &input[last_segment..]);
    Cow::Owned(decoded)
}

#[inline(always)]
fn push_no_alloc(bytes: &mut Vec<u8>, b: u8) {
    // we know that `capacity` >= `len`, so checking if they are not
    // equal is enough to ensure that we have space
    if bytes.capacity() != bytes.len() {
        bytes.push(b);
    } else {
        debug_assert!(
            false,
            "this should be unreachable -- we should have allocated enough space"
        )
    }
}

#[inline(always)]
fn extend_no_alloc(bytes: &mut Vec<u8>, slice: &[u8]) {
    if bytes.capacity().saturating_sub(bytes.len()) >= slice.len() {
        bytes.extend_from_slice(slice);
    } else {
        debug_assert!(
            false,
            "this should be unreachable -- we should have allocated enough space"
        )
    }
}
