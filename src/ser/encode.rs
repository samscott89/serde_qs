use std::borrow::Cow;

use percent_encoding::AsciiSet;

/// As defined in https://url.spec.whatwg.org/#query-percent-encode-set
///
/// The set of characters that need to be encoded in a _query_ string
/// are:
/// - CONTROL characters
/// - SPACE (but we'll separately encode it as `+`)
/// - U+0022 ("), U+0023 (#), U+003C (<), and U+003E (>).
///
/// This is the _minimal_ set of characters that need to be percent-encoded
/// in a query string.
///
/// NOTE: we add our querystring-specific characters here
/// because the encode method is only every called on
/// keys and values. This means that we _do_ want them to
/// be percent-encoded here.
const MINIMAL_QS_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    // control characters used in querystrings
    // `+` is used to represent a space in query strings
    .add(b'+')
    // denote nested keys
    .add(b'[')
    .add(b']')
    // key, value separator
    .add(b'=')
    // denote key-value pairs
    .add(b'&');

/// As defined in https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set
///
/// The application/x-www-form-urlencoded percent-encode set contains all code points, except the ASCII alphanumeric,
/// U+002A (*), U+002D (-), U+002E (.), and U+005F (_).
///
/// This is the most conservative set of characters that need to be percent-encoded.
const FORM_URLENCODED_SET: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

/// Encodes bytes for use in a querystring, applying percent-encoding as needed.
///
/// This function supports two encoding modes:
///
/// ## Query-String Encoding (default)
/// Uses the minimal WHATWG query percent-encode set, which is more permissive.
/// Spaces are encoded as `+` for better readability.
///
/// ## Form Encoding
/// Uses the stricter `application/x-www-form-urlencoded` encoding.
/// This encodes most non-alphanumeric characters, including brackets.
/// Spaces are percent-encoded as `%20`.
///
/// The function returns an iterator to avoid allocations when no encoding is needed.
pub fn encode(b: &[u8], use_form_encoding: bool) -> impl Iterator<Item = Cow<'_, [u8]>> + '_ {
    let set = if use_form_encoding {
        FORM_URLENCODED_SET
    } else {
        MINIMAL_QS_SET
    };
    percent_encoding::percent_encode(b, set).map(move |s| {
        // when using form encoding, we'll percent-encode spaces as `%20`
        // so no need to do it again
        if !use_form_encoding && s.as_bytes().contains(&b' ') {
            Cow::Owned(
                s.as_bytes()
                    .iter()
                    .map(|b| if *b == b' ' { b'+' } else { *b })
                    .collect(),
            )
        } else {
            Cow::Borrowed(s.as_bytes())
        }
    })
}
