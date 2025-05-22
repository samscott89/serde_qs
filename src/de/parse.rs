use std::borrow::Cow;
use std::iter::Iterator;
use std::slice::Iter;
use std::str;

use crate::error::{Error, Result};
use crate::map::{Entry, Map};

pub type ParsedMap<'qs> = Map<Cow<'qs, str>, ParsedValue<'qs>>;

mod decode;

/// An intermediate representation of the parsed query string.
#[derive(Debug, PartialEq)]
pub enum ParsedValue<'qs> {
    Map(Map<Cow<'qs, str>, ParsedValue<'qs>>),
    Sequence(Vec<ParsedValue<'qs>>),
    String(Cow<'qs, str>),
    Null,
    Uninitialized,
}

#[derive(Copy, Clone, Debug)]
pub struct ParsingOptions {
    pub max_depth: usize,
    pub strict: bool,
}

pub fn parse<'qs>(
    encoded_string: &'qs [u8],
    options: ParsingOptions,
) -> Result<Map<Cow<'qs, str>, ParsedValue<'qs>>> {
    let mut parser = Parser::new(encoded_string, options.max_depth, options.strict);
    let mut output = Map::default();
    parser.parse(&mut output)?;

    Ok(output)
}

/// The `Parser` struct is a stateful querystring parser.
/// It iterates over a slice of bytes, with a range to track the current
/// start/end points of a value.
struct Parser<'qs> {
    inner: &'qs [u8],
    iter: Iter<'qs, u8>,
    index: usize,
    acc: (usize, usize),
    strict: bool,
    max_depth: usize,
}

impl Parser<'_> {
    fn next(&mut self) -> Option<u8> {
        self.acc.1 = self.index;
        self.index += 1;
        let mut next = self.iter.next().copied();

        if !self.strict {
            // in non-strict mode, we will eagerly decode any bracket
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
    pub fn new(encoded: &'qs [u8], max_depth: usize, strict: bool) -> Self {
        Parser {
            inner: encoded,
            iter: encoded.iter(),
            acc: (0, 0),
            index: 0,
            strict,
            max_depth,
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
    fn collect_str(&mut self) -> Result<Option<Cow<'qs, str>>> {
        if self.acc.0 == self.acc.1 {
            // no bytes to parse
            return Ok(None);
        }
        let decoded = decode::decode(&self.inner[self.acc.0..self.acc.1], self.strict)?;
        self.clear_acc();
        Ok(Some(decoded))
    }

    /// Extracts a string from the internal byte slice from the range tracked by
    /// the parser.
    /// Avoids allocations when neither percent encoded, nor `'+'` values are
    /// present.
    fn parse_value(&mut self) -> Result<ParsedValue<'qs>> {
        self.clear_acc();
        while !matches!(self.next(), None | Some(b'&')) {
            // parse up until the '&' as the value
        }
        self.collect_str()
            .map(|v| v.map_or(ParsedValue::Null, ParsedValue::String))
    }

    /// This is the top ParsedValue parsing function. It checks the first character to
    /// decide the type of key (nested, sequence, etc.) and to call the
    /// approprate parsing function.
    ///
    /// Returns `Ok(false)` when there is no more string to parse.
    fn parse(&mut self, root_map: &mut ParsedMap<'qs>) -> Result<()> {
        let no_nesting = self.max_depth == 0;
        loop {
            let Some(x) = self.next() else {
                // we reached the end of the string
                // without encountering a terminating value (e.g. `=` or `&`)
                // nor a nested key (e.g. `[`)
                // we can insert the empty node
                // and the return `None` since there is nothing more to do
                if let Some(key) = self.collect_str()? {
                    insert_unique(self, root_map, key, ParsedValue::Null)?;
                }
                return Ok(());
            };

            // process root key
            match x {
                b'&' => {
                    // simplest case -- we have a simple key with no value
                    // insert an empty node and continue
                    let Some(key) = self.collect_str()? else {
                        // empty key -- we can skip this
                        self.clear_acc();
                        continue;
                    };
                    insert_unique(self, root_map, key, ParsedValue::Null)?;
                }
                b'=' => {
                    // we have a simple key with a value
                    // parse the value and insert it into the map
                    let key = self.collect_str()?.ok_or_else(|| {
                        // empty key
                        super::Error::parse_err("empty key", self.index)
                    })?;
                    let value = self.parse_value()?;
                    insert_unique(self, root_map, key, value)?;
                }
                b'[' if !no_nesting => {
                    // we have a nested key
                    // first get the first segment of the key
                    // and parse the rest of the key
                    let root = self.collect_str()?.ok_or_else(|| {
                        // empty key
                        super::Error::parse_err("empty key", self.index)
                    })?;
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
        let reached_max_depth = depth >= self.max_depth;
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
                ParsedValue::Map(_) | ParsedValue::String(_) | ParsedValue::Null => {
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
                        let key = self.collect_str()?.expect("key cannot be empty");
                        insert_unique(self, map, key, ParsedValue::Null)?;
                        return Ok(());
                    };

                    match b {
                        b'&' => {
                            // no value
                            let key = self.collect_str()?.expect("key cannot be empty");
                            insert_unique(self, map, key, ParsedValue::Null)?;
                        }
                        b'=' => {
                            // we have a simple key with a value
                            // parse the value and insert it into the map
                            let key = self.collect_str()?.expect("key cannot be empty");
                            let value = self.parse_value()?;
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
                        let segment = self.collect_str()?.expect("key cannot be empty");

                        // get next byte to determine next step
                        let Some(x) = self.next() else {
                            // we reached the end of the string
                            // without encountering a terminating value (e.g. `=` or `&`)
                            // nor a nested key (e.g. `[`)
                            insert_unique(self, map, segment, ParsedValue::Null)?;
                            return Ok(());
                        };
                        match x {
                            b'&' => {
                                // no value
                                insert_unique(self, map, segment, ParsedValue::Null)?;
                            }
                            b'=' => {
                                // we have a simple key with a value
                                // parse the value and insert it into the map
                                let value = self.parse_value()?;
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
    key: Cow<'qs, str>,
    value: ParsedValue<'qs>,
) -> Result<()> {
    match map.entry(key) {
        Entry::Occupied(o) => {
            return Err(Error::parse_err(
                format!("Multiple values for the same key: {}", o.key()),
                parser.index,
            ));
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
            let value = parser.parse_value()?;
            seq.push(value);
        }
        Some(b'&') => {
            // No value
            seq.push(ParsedValue::Null);
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
            seq.push(ParsedValue::Null);
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
        ParsedValue::Null => Err(super::Error::parse_err(
            "invalid input: the same key is used for both a unit value and a nested map",
            0,
        )),
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;

    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_CONFIG: ParsingOptions = ParsingOptions {
        max_depth: 10,
        strict: false,
    };

    #[test]
    fn parse_empty() {
        let parsed = parse(b"", TEST_CONFIG).unwrap();
        assert_eq!(parsed, Default::default())
    }

    #[test]
    fn parse_map() {
        let parsed = parse(b"abc=def", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([("abc".into(), ParsedValue::String("def".into()))])
        );
    }

    #[test]
    fn parse_map_no_value() {
        let parsed = parse(b"abc", TEST_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("abc".into(), ParsedValue::Null)]));
    }

    #[test]
    fn parse_map_empty_value() {
        let parsed = parse(b"abc=", TEST_CONFIG).unwrap();
        assert_eq!(parsed, Map::from_iter([("abc".into(), ParsedValue::Null)]));
    }

    #[test]
    fn parse_sequence() {
        let parsed = parse(b"abc[]=1&abc[]=2", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            // NOTE: we cannot have a top-level sequence since we need a key to group
            // the values by
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Sequence(vec![
                    ParsedValue::String("1".into()),
                    ParsedValue::String("2".into())
                ])
            )])
        );
    }

    #[test]
    fn parse_ordered_sequence() {
        let parsed = parse(b"abc[1]=1&abc[0]=0", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Map(Map::from_iter([
                    ("1".into(), ParsedValue::String("1".into())),
                    ("0".into(), ParsedValue::String("0".into()))
                ]))
            )])
        );
    }

    #[test]
    fn parse_nested_map() {
        let parsed = parse(b"abc[def]=ghi", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([(
                "abc".into(),
                ParsedValue::Map(Map::from_iter([(
                    "def".into(),
                    ParsedValue::String("ghi".into())
                )]))
            )])
        );
    }

    #[test]
    fn parse_empty_and_sequence() {
        let parse_err = parse(b"abc&abc[]=1", TEST_CONFIG).unwrap_err();
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
        let parsed = parse(b"e[B]&v[V1][x]=12&v[V1][y]=300&u=12", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([
                (
                    "e".into(),
                    ParsedValue::Map(Map::from_iter([("B".into(), ParsedValue::Null)]))
                ),
                ("u".into(), ParsedValue::String("12".into())),
                (
                    "v".into(),
                    ParsedValue::Map(Map::from_iter([(
                        "V1".into(),
                        ParsedValue::Map(Map::from_iter([
                            ("x".into(), ParsedValue::String("12".into())),
                            ("y".into(), ParsedValue::String("300".into()))
                        ]))
                    )]))
                ),
            ])
        );
    }

    #[test]
    fn parse_strict() {
        let parsed = parse(
            b"a[b][c][d][e][f][g][h]=i",
            ParsingOptions {
                max_depth: 5,
                strict: false,
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
                                        ParsedValue::String("i".into())
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
    fn parse_encoded_brackets() {
        let strict_config = ParsingOptions {
            max_depth: 10,
            strict: true,
        };
        // // encoded in the key
        // // in strict mode, the brackets are not decoded
        // let parsed = parse(b"abc%5Bdef%5D=ghi", strict_config).unwrap();
        // assert_eq!(
        //     parsed,
        //     Map::from_iter([("abc[def]".into(), ParsedValue::String("ghi".into()))])
        // );

        // // encoded in the key
        // // in non-strict mode, the brackets are eagerly decoded
        // let parsed = parse(b"abc%5Bdef%5D=ghi", TEST_CONFIG).unwrap();
        // assert_eq!(
        //     parsed,
        //     Map::from_iter([(
        //         "abc".into(),
        //         ParsedValue::Map(Map::from_iter([(
        //             "def".into(),
        //             ParsedValue::String("ghi".into())
        //         )]))
        //     )])
        // );

        // encoded in the value
        let parsed = parse(b"foo=%5BHello%5D", strict_config).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([("foo".into(), ParsedValue::String("[Hello]".into()))])
        );
        // same result in non-strict mode
        let parsed = parse(b"foo=%5BHello%5D", TEST_CONFIG).unwrap();
        assert_eq!(
            parsed,
            Map::from_iter([("foo".into(), ParsedValue::String("[Hello]".into()))])
        );
    }
}
