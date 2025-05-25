use std::borrow::Cow;
use std::iter::Iterator;
use std::slice::Iter;
use std::{fmt, str};

use serde::de::IntoDeserializer;

use crate::error::{Error, Result};
use crate::map::{Entry, Map};

use super::string_parser::StringParsingDeserializer;

pub type ParsedMap<'qs> = Map<Key<'qs>, ParsedValue<'qs>>;

mod decode;

/// Represents a key in the parsed querystring.
///
/// Keys can be either integers (for array indices) or strings (for object keys).
/// This allows the parser to handle both `items[0]=foo` (integer key) and
/// `user[name]=bar` (string key) notations.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key<'a> {
    Int(usize),
    String(Cow<'a, [u8]>),
}

impl Key<'_> {
    /// In some cases, we would rather push an empty key
    /// (e.g. if we have `foo=1&=2`, then we'll have a map `{ "foo": 1, "": 2 }`).
    fn empty_key() -> Self {
        Key::String(Cow::Borrowed(b""))
    }
}

impl fmt::Debug for Key<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Int(i) => write!(f, "{i}"),
            Key::String(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),
        }
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Key<'a> {
    fn from(s: &'a [u8]) -> Self {
        Key::String(Cow::Borrowed(s))
    }
}

impl From<usize> for Key<'_> {
    fn from(i: usize) -> Self {
        Key::Int(i)
    }
}

impl<'a> Key<'a> {
    pub fn deserialize_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'a>,
    {
        match self {
            Key::Int(i) => seed.deserialize(i.into_deserializer()),
            Key::String(s) => seed.deserialize(StringParsingDeserializer::new(s)?),
        }
    }
}

/// An intermediate representation of the parsed query string.
///
/// This enum represents the different types of values that can appear in a querystring.
/// The parser builds a tree of these values before the final deserialization step.
///
/// - `Map`: Nested objects like `user[name]=John&user[age]=30`
/// - `Sequence`: Arrays like `ids[0]=1&ids[1]=2`
/// - `String`: Leaf values containing the actual data
/// - `Null`: Empty values like `key=` or standalone keys like `flag`
/// - `Uninitialized`: Used internally during parsing for placeholder values
#[derive(PartialEq)]
pub enum ParsedValue<'qs> {
    Map(ParsedMap<'qs>),
    Sequence(Vec<ParsedValue<'qs>>),
    String(Cow<'qs, [u8]>),
    /// Null value means we have a key with an _empty_ value string
    /// e.g. `"key"=`
    Null,
    /// NoValue means we have a key with no value at all, e.g. `"key"`
    NoValue,
    Uninitialized,
}

impl fmt::Debug for ParsedValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedValue::Map(m) => f.debug_map().entries(m.iter()).finish(),
            ParsedValue::Sequence(s) => f.debug_list().entries(s.iter()).finish(),
            ParsedValue::String(s) => write!(f, "String({})", String::from_utf8_lossy(s)),
            ParsedValue::Null => write!(f, "Null"),
            ParsedValue::NoValue => write!(f, "NoValue"),
            ParsedValue::Uninitialized => write!(f, "Unintialized"),
        }
    }
}

pub fn parse(encoded_string: &[u8], config: crate::Config) -> Result<ParsedMap<'_>> {
    let mut parser = Parser::new(encoded_string, config);
    let mut output = Map::default();
    parser.parse(&mut output)?;

    Ok(output)
}

/// The `Parser` struct is a stateful querystring parser.
///
/// It iterates over a slice of bytes, maintaining an accumulator range `(start, end)`
/// to track the current segment being parsed. This approach avoids allocations
/// by working directly with slices of the input string.
///
/// The parser handles bracket notation for nested structures and supports both
/// query-string encoding and form encoding modes.
struct Parser<'qs> {
    inner: &'qs [u8],
    iter: Iter<'qs, u8>,
    index: usize,
    acc: (usize, usize),
    config: crate::Config,
}

impl Parser<'_> {
    fn next(&mut self) -> Option<u8> {
        self.acc.1 = self.index;
        self.index += 1;
        let mut next = self.iter.next().copied();

        if self.config.use_form_encoding {
            // in formencoding mode, we will eagerly decode any
            // percent-encoded brackets
            if matches!(next, Some(b'%')) {
                let iter = self.iter.as_slice();
                if iter.len() >= 2 {
                    match &self.iter.as_slice()[..2] {
                        b"5B" => {
                            // skip the next two characters
                            let _ = self.iter.next();
                            let _ = self.iter.next();
                            self.index += 2;
                            next = Some(b'[');
                        }
                        b"5D" => {
                            // skip the next two characters
                            let _ = self.iter.next();
                            let _ = self.iter.next();
                            self.index += 2;
                            next = Some(b']');
                        }
                        _ => {
                            // unknown percent encoding, leave it as is
                        }
                    }
                }
            }
        }
        next
    }
}

impl<'qs> Parser<'qs> {
    pub fn new(encoded: &'qs [u8], config: crate::Config) -> Self {
        Parser {
            inner: encoded,
            iter: encoded.iter(),
            acc: (0, 0),
            index: 0,
            config,
        }
    }

    /// Resets the accumulator range by setting `(start, end)` to `(end, end)`.
    fn clear_acc(&mut self) {
        self.acc = (self.index, self.index);
    }

    /// Extracts a string from the internal byte slice from the range tracked by
    /// the parser.
    /// Avoids allocations when neither percent encoded, nor `'+'` values are
    /// present.
    fn collect_key(&mut self) -> Result<Option<Key<'qs>>> {
        if self.acc.0 == self.acc.1 {
            // no bytes to parse
            return Ok(None);
        }
        let bytes = &self.inner[self.acc.0..self.acc.1];
        if bytes.iter().all(|b| b.is_ascii_digit()) {
            // if all bytes are digits, we can parse it as an integer
            // SAFETY: we know that all bytes are ASCII digits
            let key = unsafe { std::str::from_utf8_unchecked(bytes) };
            if let Ok(key) = key.parse::<usize>() {
                self.clear_acc();
                return Ok(Some(Key::Int(key)));
            }
            // if this fails, we'll just fall back to the string case
        }
        let string_key = Key::String(decode::decode(bytes));
        self.clear_acc();
        Ok(Some(string_key))
    }

    /// Extracts a string from the internal byte slice from the range tracked by
    /// the parser.
    /// Avoids allocations when neither percent encoded, nor `'+'` values are
    /// present.
    fn collect_value(&mut self) -> Result<ParsedValue<'qs>> {
        // clear the accumulator to start fresh
        self.clear_acc();
        while !matches!(self.next(), None | Some(b'&')) {
            // eat bytes up until the next '&' (or end of string) as the value
        }

        if self.acc.0 == self.acc.1 {
            // no bytes to parse
            return Ok(ParsedValue::Null);
        }

        let decoded = decode::decode(&self.inner[self.acc.0..self.acc.1]);
        self.clear_acc();
        Ok(ParsedValue::String(decoded))
    }

    /// Main parsing entry point that processes the querystring into a map structure.
    ///
    /// This function handles the top-level parsing logic, identifying key-value pairs
    /// and delegating to specialized parsing functions for nested structures.
    /// It processes the input byte-by-byte, handling special characters like
    /// `&` (pair separator), `=` (key-value separator), and `[`/`]` (nesting).
    fn parse(&mut self, root_map: &mut ParsedMap<'qs>) -> Result<()> {
        if self.inner.is_empty() {
            // empty string -- nothing to parse
            return Ok(());
        }
        let no_nesting = self.config.max_depth == 0;
        loop {
            let Some(x) = self.next() else {
                // we reached the end of the string
                // push the key (if exists) with a null value
                if let Some(key) = self.collect_key()? {
                    insert_unique(self, root_map, key, ParsedValue::NoValue)?;
                }

                // we've finished parsing the string
                return Ok(());
            };

            // process root key
            match x {
                b'&' => {
                    // we have a simple key with no value
                    // insert an empty node and continue
                    if let Some(key) = self.collect_key()? {
                        // if we have no key, we'll skip it entirely to avoid creating empty
                        // key, value pairs
                        insert_unique(self, root_map, key, ParsedValue::NoValue)?;
                    }
                }
                b'=' => {
                    // we have a simple key with a value
                    // parse the value and insert it into the map
                    // if they key is empty, since we have an explicit `=` we'll use
                    // an empty key
                    let key = self.collect_key()?.unwrap_or_else(Key::empty_key);
                    let value = self.collect_value()?;
                    insert_unique(self, root_map, key, value)?;
                }
                b'[' if !no_nesting => {
                    // we have a nested key
                    // first get the first segment of the key
                    // and parse the rest of the key
                    let root = self.collect_key()?.unwrap_or_else(Key::empty_key);
                    let node = root_map.entry(root).or_insert(ParsedValue::Uninitialized);
                    // parse the key and insert it into the map
                    self.parse_nested_key(node, 0)?;
                }
                _ => {
                    // for any other character
                    // do nothing, keep accumulating the key
                    continue;
                }
            }

            // if we reached here we pushed a new value -- clear the accumulator
            self.clear_acc();
        }
    }

    fn parse_nested_key(
        &mut self,
        current_node: &mut ParsedValue<'qs>,
        depth: usize,
    ) -> Result<()> {
        let reached_max_depth = depth >= self.config.max_depth;
        if !reached_max_depth {
            // if we haven't reached the maximum depth yet, we can clear the accumulator
            // otherwise, we want to keep the accumulated `[` character
            self.clear_acc();
        }

        let Some(first_byte) = self.next() else {
            return Err(super::Error::parse_err(
                "query string ended before expected",
                self.index,
            ));
        };

        if first_byte == b']' {
            // empty key (e.g. "[]") -- parse as a sequence
            match current_node {
                ParsedValue::Sequence(seq) => {
                    parse_sequence_value(self, seq)?;
                }
                ParsedValue::Uninitialized => {
                    // initialize this node as a sequence
                    let mut seq = vec![];
                    parse_sequence_value(self, &mut seq)?;
                    *current_node = ParsedValue::Sequence(seq);
                }
                ParsedValue::Map(_)
                | ParsedValue::String(_)
                | ParsedValue::Null
                | ParsedValue::NoValue => {
                    return Err(super::Error::parse_err(
                        "invalid input: the same key is used for both a value and a sequence",
                        self.index,
                    ));
                }
            }
        } else {
            // otherwise we have a key
            // and this entry _must_ be a map
            let map = expect_map(self, current_node)?;

            if reached_max_depth {
                // if we've reached the maximum depth already, we'll just parse the entire
                // key as a string and insert it into the map
                loop {
                    let Some(b) = self.next() else {
                        // we've reached the end of the string
                        // without encountering a terminating value (e.g. `=` or `&`)
                        let key = self.collect_key()?.expect("key cannot be empty");
                        insert_unique(self, map, key, ParsedValue::NoValue)?;
                        return Ok(());
                    };

                    match b {
                        b'&' => {
                            // no value
                            let key = self.collect_key()?.expect("key cannot be empty");
                            insert_unique(self, map, key, ParsedValue::NoValue)?;
                        }
                        b'=' => {
                            // we have a simple key with a value
                            // parse the value and insert it into the map
                            let key = self.collect_key()?.expect("key cannot be empty");
                            let value = self.collect_value()?;
                            insert_unique(self, map, key, value)?;
                        }
                        _ => {
                            // otherwise, continue parsing the key
                            continue;
                        }
                    }
                    break;
                }
            } else {
                // parse until the closing bracket
                loop {
                    let Some(b) = self.next() else {
                        return Err(super::Error::parse_err(
                            "unexpected end of input while parsing nested key",
                            self.index,
                        ));
                    };

                    if b == b']' {
                        // finished parsing the key
                        let segment = self.collect_key()?.expect("key cannot be empty");

                        // get next byte to determine next step
                        let Some(x) = self.next() else {
                            // we reached the end of the string
                            // without encountering a terminating value (e.g. `=` or `&`)
                            // nor a nested key (e.g. `[`)
                            insert_unique(self, map, segment, ParsedValue::NoValue)?;
                            return Ok(());
                        };
                        match x {
                            b'&' => {
                                // no value
                                insert_unique(self, map, segment, ParsedValue::NoValue)?;
                            }
                            b'=' => {
                                // we have a simple key with a value
                                // parse the value and insert it into the map
                                let value = self.collect_value()?;
                                insert_unique(self, map, segment, value)?;
                            }
                            b'[' => {
                                // we have a nested key
                                let node = map.entry(segment).or_insert(ParsedValue::Uninitialized);
                                // parse the key and insert it into the map
                                self.parse_nested_key(node, depth + 1)?;
                            }
                            _ => {
                                let char = x as char;
                                return Err(super::Error::parse_err(
                                    format!("unexpected character `{char}` while parsing nested key: expected `&`, `=` or `[`"),
                                    self.index,
                                ));
                            }
                        }
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

fn insert_unique<'qs>(
    parser: &mut Parser<'_>,
    map: &mut ParsedMap<'qs>,
    key: Key<'qs>,
    value: ParsedValue<'qs>,
) -> Result<()> {
    match map.entry(key) {
        Entry::Occupied(mut o) => {
            let entry = o.get_mut();
            match entry {
                ParsedValue::Map(_) => {
                    return Err(Error::parse_err(
                        format!("Multiple values for the same key: {}", o.key()),
                        parser.index,
                    ));
                }
                ParsedValue::Sequence(parsed_values) => {
                    // if the value is a sequence, we can just push the new value
                    parsed_values.push(value);
                    return Ok(());
                }
                ParsedValue::String(_) => {
                    // we'll support mutliple values for the same key
                    // by converting the existing value into a sequence
                    // and pushing the new value into it
                    // later we'll handle this case by taking the last value of
                    // the sequence
                    let existing = std::mem::replace(entry, ParsedValue::Uninitialized);
                    let mut seq = vec![existing];
                    seq.push(value);
                    *entry = ParsedValue::Sequence(seq);
                }
                ParsedValue::NoValue | ParsedValue::Null => {
                    return Err(Error::parse_err(
                        format!("Multiple values for the same key: {}", o.key()),
                        parser.index,
                    ));
                }
                ParsedValue::Uninitialized => {
                    return Err(Error::parse_err(
                        format!("internal error: value is unintialized: {}", o.key()),
                        parser.index,
                    ));
                }
            }
        }
        Entry::Vacant(v) => {
            v.insert(value);
        }
    }
    Ok(())
}

fn parse_sequence_value<'qs>(
    parser: &mut Parser<'qs>,
    seq: &mut Vec<ParsedValue<'qs>>,
) -> Result<()> {
    match parser.next() {
        Some(b'=') => {
            // Key is finished, parse up until the '&' as the value
            let value = parser.collect_value()?;
            seq.push(value);
        }
        Some(b'&') => {
            // No value
            seq.push(ParsedValue::NoValue);
        }
        Some(b'[') => {
            // we cannot handle unindexed sequences of maps
            // since we would have parsing ambiguity
            // e.g. `abc[][x]=1&abc[][y]=2`
            // could either be two entries with `x` and `y` set alternatively
            // or a single entry with both set
            return Err(super::Error::parse_err(
                "unsupported: unable to parse nested maps of unindexed sequences ",
                parser.index,
            ));
        }
        None => {
            // The string has ended, so the value is empty.
            seq.push(ParsedValue::NoValue);
        }
        _ => {
            return Err(super::Error::parse_err(
                        "unsupported: cannot mix unindexed sequences `abc[]=...` with indexed sequences `abc[0]=...`",
                        parser.index,
                    ));
        }
    }
    Ok(())
}

fn expect_map<'a, 'qs>(
    parser: &mut Parser<'qs>,
    node: &'a mut ParsedValue<'qs>,
) -> Result<&'a mut ParsedMap<'qs>> {
    match node {
        ParsedValue::Map(map) => Ok(map),
        ParsedValue::Uninitialized => {
            *node = ParsedValue::Map(Map::default());
            if let ParsedValue::Map(ref mut map) = *node {
                Ok(map)
            } else {
                unreachable!()
            }
        }
        ParsedValue::Sequence(_) => Err(super::Error::parse_err(
            "invalid input: the same key is used for both a sequence and a nested map",
            parser.index,
        )),
        ParsedValue::String(_) => Err(super::Error::parse_err(
            "invalid input: the same key is used for both a value and a nested map",
            0,
        )),
        ParsedValue::NoValue | ParsedValue::Null => Err(super::Error::parse_err(
            "invalid input: the same key is used for both a unit value and a nested map",
            0,
        )),
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, iter::FromIterator};

    use crate::Config;

    use super::{parse, ParsedValue};

    use pretty_assertions::assert_eq;

    type Map<'a> = super::ParsedMap<'a>;

    static DEFAULT_CONFIG: Config = Config {
        max_depth: 10,
        use_form_encoding: false,
    };
    static FORM_ENCODING_CONFIG: Config = Config {
        use_form_encoding: true,
        ..DEFAULT_CONFIG
    };

    impl<'a> From<&'a str> for ParsedValue<'a> {
        fn from(s: &'a str) -> Self {
            ParsedValue::String(Cow::Borrowed(s.as_bytes()))
        }
    }

    #[test]
    fn parse_empty() {
        let parsed = parse(b"", DEFAULT_CONFIG).unwrap();
        assert_eq!(parsed, Map::default())
    }

    #[test]
    fn parse_map() {
        let parsed = parse(b"abc=def", DEFAULT_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("abc".into(), "def".into())]));
    }

    #[test]
    fn parse_map_no_value() {
        let parsed = parse(b"abc", DEFAULT_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([("abc".into(), ParsedValue::NoValue)])
        );
    }

    #[test]
    fn parse_map_null_value() {
        let parsed = parse(b"abc=", DEFAULT_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("abc".into(), ParsedValue::Null)]));
    }

    #[test]
    fn parse_sequence() {
        let parsed = parse(b"abc[]=1&abc[]=2", DEFAULT_CONFIG).unwrap();
        assert_eq!(
            parsed,
            // NOTE: we cannot have a top-level sequence since we need a key to group
            // the values by
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Sequence(vec!["1".into(), "2".into()])
            )])
        );
    }

    #[test]
    fn parse_ordered_sequence() {
        let parsed = parse(b"abc[1]=1&abc[0]=0", DEFAULT_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Map(Map::from_iter([
                    (1.into(), "1".into()),
                    (0.into(), "0".into())
                ]))
            )])
        );
    }

    #[test]
    fn parse_nested_map() {
        let parsed = parse(b"abc[def]=ghi", DEFAULT_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Map(Map::from_iter([("def".into(), "ghi".into())]))
            )])
        );
    }

    #[test]
    fn parse_empty_and_sequence() {
        let parse_err = parse(b"abc&abc[]=1", DEFAULT_CONFIG).unwrap_err();
        assert!(
            parse_err
                .to_string()
                .contains("invalid input: the same key is used for both a value and a sequence"),
            "got: {}",
            parse_err
        );
    }

    #[test]
    fn parse_many() {
        let parsed = parse(b"e[B]&v[V1][x]=12&v[V1][y]=300&u=12", DEFAULT_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([
                (
                    "e".into(),
                    ParsedValue::Map(Map::from_iter([("B".into(), ParsedValue::NoValue)]))
                ),
                ("u".into(), "12".into()),
                (
                    "v".into(),
                    ParsedValue::Map(Map::from_iter([(
                        "V1".into(),
                        ParsedValue::Map(Map::from_iter([
                            ("x".into(), "12".into()),
                            ("y".into(), "300".into())
                        ]))
                    )]))
                ),
            ])
        );
    }

    #[test]
    fn parse_max_depth() {
        let parsed = parse(
            b"a[b][c][d][e][f][g][h]=i",
            Config {
                max_depth: 5,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(
            parsed,
            Map::from_iter([(
                "a".into(),
                ParsedValue::Map(Map::from_iter([(
                    "b".into(),
                    ParsedValue::Map(Map::from_iter([(
                        "c".into(),
                        ParsedValue::Map(Map::from_iter([(
                            "d".into(),
                            ParsedValue::Map(Map::from_iter([(
                                "e".into(),
                                ParsedValue::Map(Map::from_iter([(
                                    "f".into(),
                                    ParsedValue::Map(Map::from_iter([(
                                        "[g][h]".into(),
                                        "i".into()
                                    )]))
                                )]))
                            )]))
                        )]))
                    )]))
                )]))
            )])
        );
    }

    #[test]
    fn parse_formencoded_brackets() {
        // encoded in the key
        // in non-strict mode, the brackets are eagerly decoded
        let parsed = parse(b"abc%5Bdef%5D=ghi", FORM_ENCODING_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Map(Map::from_iter([("def".into(), "ghi".into())]))
            )])
        );

        let parsed = parse(b"foo=%5BHello%5D", FORM_ENCODING_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("foo".into(), "[Hello]".into())]));
    }

    #[test]
    fn parse_encoded_brackets() {
        // encoded in the key
        // in strict mode, the brackets are not decoded, so we end up with a key containing
        // brackets
        let parsed = parse(b"abc%5Bdef%5D=ghi", DEFAULT_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("abc[def]".into(), "ghi".into())]));

        // encoded in the value
        let parsed = parse(b"foo=%5BHello%5D", DEFAULT_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("foo".into(), "[Hello]".into())]));
    }
}
