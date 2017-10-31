use percent_encoding;
use serde::de;

use std::borrow::Cow;
use std::slice::Iter;

use super::*;

/// To override the default serialization parameters, first construct a new
/// Config.
///
/// A `max_depth` of 0 implies no nesting: the result will be a flat map.
/// This is mostly useful when the maximum nested depth is known beforehand,
/// to prevent denial of service attacks by providing incredibly deeply nested
/// inputs.
///
/// The default value for `max_depth` is 5.
///
/// ```
/// use serde_qs::Config;
/// use std::collections::HashMap;
///
/// let config = Config::with_max_depth(0);
/// let map: HashMap<String, String> = config.deserialize_str("a[b][c]=1")
///                                          .unwrap();
/// assert_eq!(map.get("a[b][c]").unwrap(), "1");
///
/// let config = Config::with_max_depth(10);
/// let map: HashMap<String, HashMap<String, HashMap<String, String>>> =
///             config.deserialize_str("a[b][c]=1").unwrap();
/// assert_eq!(map.get("a").unwrap().get("b").unwrap().get("c").unwrap(), "1");
/// ```
///
pub struct Config {
    /// Specifies the maximum depth key that `serde_qs` will attempt to
    /// deserialize. Default is 5.
    max_depth: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { max_depth: 5 }
    }
}

impl Config {
    /// Construct a new `Config` with the specified maximum depth of nesting.
    pub fn with_max_depth(depth: usize) -> Config {
        Config {
            max_depth: depth
        }
    }

    /// Get maximum depth parameter.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Config {
    /// Deserializes a querystring from a `&[u8]` using this `Config`.
    pub fn deserialize_bytes<'de, T: de::Deserialize<'de>>(&self,
                                                 input: &'de [u8])
                                                 -> Result<T> {
        T::deserialize(QsDeserializer::with_config(self, input))
    }

    /// Deserializes a querystring from a `&str` using this `Config`.
    pub fn deserialize_str<'de, T: de::Deserialize<'de>>(&self,
                                               input: &'de str)
                                               -> Result<T> {
        self.deserialize_bytes(input.as_bytes())
    }
}


macro_rules! tu {
    ($x:expr) => (
        match $x {
            Some(x) => *x,
            None => return Err(
                de::Error::custom("query string ended before expected"))
        }
    )
}

impl<'a> Level<'a> {
    /// If this `Level` value is indeed a map, then attempt to insert
    /// `value` for key `key`.
    /// Returns error if `self` is not a map, or already has an entry for that
    /// key.
    fn insert_map_value(&mut self, key: Cow<'a, str>, value: Cow<'a, str>) {
        if let Level::Nested(ref mut map) = *self {
            match map.entry(key) {
                Entry::Occupied(mut o) => {
                    // Throw away old result; map is now invalid anyway.
                    let _ = o.insert(Level::Invalid("Multiple values for one key"));
                },
                Entry::Vacant(vm) => {
                    // Map is empty, result is None
                    let _ = vm.insert(Level::Flat(value));
                },
            }
        } else if let Level::Uninitialised = *self  {
            let mut map = BTreeMap::default();
            let _ = map.insert(key, Level::Flat(value));
            *self = Level::Nested(map);
        } else {
            *self = Level::Invalid("Attempted to insert map value into non-map structure");
        }
    }

    /// If this `Level` value is indeed a seq, then attempt to insert
    /// `value` for key `key`.
    /// Returns error if `self` is not a seq, or already has an entry for that
    /// key.
    fn insert_ord_seq_value(&mut self, key: usize, value: Cow<'a, str>) {
        if let Level::OrderedSeq(ref mut map) = *self {
            match map.entry(key) {
                Entry::Occupied(mut o) => {
                    // Throw away old result; map is now invalid anyway.
                    let _ = o.insert(Level::Invalid("Multiple values for one key"));
                },
                Entry::Vacant(vm) => {
                    // Map is empty, result is None
                    let _ = vm.insert(Level::Flat(value));
                },
            }
        } else if let Level::Uninitialised = *self  {
            // To reach here, self is either an OrderedSeq or nothing.
            let mut map = BTreeMap::default();
            let _ = map.insert(key, Level::Flat(value));
            *self = Level::OrderedSeq(map);
        } else {
            *self = Level::Invalid("Attempted to insert seq value into non-seq structure");
        }
    }
}

use std::iter::Iterator;
use std::str;

pub struct Parser<'a> {
    inner: &'a [u8],
    iter: Iter<'a, u8>,
    // `acc` stores an index range for the current value
    acc: (usize, usize),
    peeked: Option<&'a u8>,
    depth: usize,
}

use std::fmt;
impl<'a> fmt::Debug for Parser<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parser\n\tinner: {}\n\tcurrent: {:?}\n\tpeeked: {:?}", 
            String::from_utf8_lossy(self.inner),
            self.acc,
            // String::from_utf8_lossy(&self.inner[self.acc.0..self.acc.1 - 1]),
            self.peeked
        )
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = &'a u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(v) => Some(v),
            None => {
                self.acc.1 += 1;
                self.iter.next()
            }
        }
    }
}


/// Replace b'+' with b' '
/// Copied from `form_urlencoded`
fn replace_plus<'a>(input: Cow<'a, str>) -> Cow<'a, str> {
    match input.as_bytes().iter().position(|&b| b == b'+') {
        None => input,
        Some(first_position) => {
            let mut replaced = input.as_bytes().to_owned();
            replaced[first_position] = b' ';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b'+' {
                    *byte = b' ';
                }
            }
            Cow::Owned(String::from_utf8(replaced).expect("replacing '+' with ' ' cannot panic"))
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(encoded: &'a [u8], depth: usize) -> Self {
        Parser {
            inner: encoded, 
            iter: encoded.iter(),
            acc: (0, 0),
            peeked: None,
            depth: depth,
        }
    }

    fn clear_acc(&mut self) {
        self.acc.0 = self.acc.1;
    }

    fn decode_acc(&mut self) -> Result<Cow<'a, str>> {
        let res: Cow<'a, str> = percent_encoding::percent_decode(&self.inner[self.acc.0..self.acc.1 - 1]).decode_utf8()?;
        let res: Result<Cow<'a, str>> = Ok(replace_plus(res));
        println!("({}, {})", self.acc.0, self.acc.1);
        self.clear_acc();
        println!("Decoded: {:?}", res);
        res.map_err(Error::from)
    }

    pub fn as_deserializer(&mut self) -> QsDeserializer<'a> {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);
        while let Ok(x) = self.parse(&mut root) {
            if !x {
                break;
            }
        }
        let iter = match root {
            Level::Nested(map) => map.into_iter(),
            _ => BTreeMap::default().into_iter()
        };
        QsDeserializer {
            iter: iter,
            value: None,
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.peeked.is_some() {
            self.peeked
        } else if let Some(x) = self.next() {
            self.peeked = Some(x);
            Some(x)
        } else {
            None
        }
    }

    fn parse_key(&mut self,
                 end_on: u8,
                 consume: bool)
                 -> Result<Cow<'a, str>> {
        loop {
            if let Some(x) = self.next() {
                match *x {
                    c if c == end_on => {
                        // Add this character back to the buffer for peek.
                        if !consume {
                            self.peeked = Some(x);
                        }

                        return self.decode_acc().map_err(Error::from);
                    },
                    b'=' => {
                        // Allow the '=' byte only when parsing keys within []
                        if end_on != b']' {
                            let res = self.decode_acc();
                            // Add this character back to the buffer for peek.
                            self.peeked = Some(x);

                            return res.map_err(Error::from);
                        }
                    },

                    b'&' => {
                        let res = self.decode_acc();
                        // let res = String::from_utf8(self.acc.split_off(0));
                        // self.acc.clear();
                        // self.acc.push(b'&');
                        // self.acc.
                        self.peeked = Some(&b'&');
                        return res.map_err(Error::from);
                    },
                    // x @ 0x20...0x7e | x @ ' ' => {
                    //     self.acc.push(x);
                    // },
                    _ => {
                        // do nothing, keep adding to key
                        // return Err(de::Error::custom("unexpected character \
                        //                               in query string."));
                    },
                }
            } else {
                let res = self.decode_acc();
                // self.acc.clear();
                return res.map_err(Error::from);
            }
        }
    }

    fn parse_map_value(&mut self,
                       key: Cow<'a, str>,
                       node: &mut Level<'a>)
                       -> Result<()> {
        let res = if let Some(x) = self.peek() {
            match *x {
                b'=' => {
                    self.clear_acc();
                    for _ in self.take_while(|b| *b != &b'&') {}
                    let value: Cow<'a, str> = self.decode_acc()?;
                    // Reached the end of the key string
                    node.insert_map_value(key, value);
                    Ok(())
                },
                b'&' => {
                    node.insert_map_value(key, Cow::Borrowed(""));
                    Ok(())
                },
                b'[' => {
                    if let Level::Uninitialised = *node {
                        *node = Level::Nested(BTreeMap::default());
                    }
                    if let Level::Nested(ref mut map) = *node {
                        self.depth -= 1;
                        let _ = self.parse(map.entry(key)
                                .or_insert(Level::Uninitialised))?;
                        Ok(())
                    } else {
                        Err(de::Error::custom(format!("tried to insert a \
                                                       new key into {:?}",
                                                      node)))
                    }
                },
                _ => {
                    Err(de::Error::custom("Unexpected character found when parsing"))
                },
            }
        } else {
            node.insert_map_value(key, Cow::Borrowed(""));
            Ok(())
        };
        self.depth +=1;
        res
    }

    fn parse_seq_value(&mut self, node: &mut Level<'a>) -> Result<()> {
        let res = match tu!(self.peek()) {
            b'=' => {
                self.clear_acc();
                // Iterate through until finding '&' character.
                for _ in self.take_while(|b| *b != &b'&') {}
                let value = self.decode_acc()?;
                // Reached the end of the key string
                if let Level::Sequence(ref mut seq) = *node {
                    seq.push(Level::Flat(value));
                } else {
                    let mut seq = Vec::new();
                    seq.push(Level::Flat(value));
                    *node = Level::Sequence(seq);
                }
                Ok(())
            },
            _ => {
                Err(de::Error::custom("non-indexed sequence of structs not \
                                       supported"))
            },
        };
        self.depth += 1;
        res
    }

    fn parse_ord_seq_value(&mut self, key: usize, node: &mut Level<'a>) -> Result<()> {
        let res = if let Some(x) = self.peek() {
            match *x {
                b'=' => {
                    self.clear_acc();
                    // Iterate through until finding '&' character.
                    for _ in self.take_while(|b| *b != &b'&') {}
                    let value = self.decode_acc()?;
                    // Reached the end of the key string
                    node.insert_ord_seq_value(key, value);
                    Ok(())
                },
                b'&' => {
                    node.insert_ord_seq_value(key, Cow::Borrowed(""));
                    Ok(())
                },
                b'[' => {
                    if let Level::Uninitialised = *node {
                        *node = Level::OrderedSeq(BTreeMap::default());
                    }
                    if let Level::OrderedSeq(ref mut map) = *node {
                        self.depth -= 1;
                        let _ = self.parse(map.entry(key)
                                .or_insert(Level::Uninitialised))?;
                        Ok(())
                    } else {
                        Err(de::Error::custom(format!("tried to insert a \
                                                       new key into {:?}",
                                                      node)))
                    }
                },
                _ => {
                    Err(de::Error::custom("Unexpected character found when parsing"))
                },
            }
        } else {
            node.insert_ord_seq_value(key, Cow::Borrowed(""));
            Ok(())
        };
        self.depth += 1;
        res
    }


    fn parse(&mut self, node: &mut Level<'a>) -> Result<bool> {
        // First character determines parsing type
        if self.depth == 0 {
            // Hit the maximum depth level, so parse everything as a key
            let key = self.parse_key(b'\x00', true)?;
            self.parse_map_value(key, node)?;
            // self.depth += 1;
            return Ok(true);
        }
        // println!("Beginning new parse\n{:?}", self);
        match self.next() {
            Some(x) => {
                match *x {
                    b'[' => {
                        self.clear_acc();
                        println!("Parsing nested key: \n{:?}", self);
                        match tu!(self.peek()) {
                            // key is of the form "[...", not really allowed.
                            b'[' => {
                                Err(de::Error::custom("found another opening bracket before the closed bracket"))

                            },
                            // key is simply "[]", so treat as a seq.
                            b']' => {
                                // throw away the bracket 
                                let _ = self.next();
                                self.clear_acc();
                                self.parse_seq_value(node)?;
                                // self.depth += 1;
                                Ok(true)

                            },
                            // First character is an integer, attempt to parse it as an integer key
                            b'0'...b'9' => {
                                let key = self.parse_key(b']', true)?;
                                let key = usize::from_str_radix(&key, 10).map_err(Error::from)?;
                                self.parse_ord_seq_value(key, node)?;
                                // self.depth += 1;
                                Ok(true)
                            }
                            // Key is "[a..." so parse up to the closing "]"
                            0x20...0x2f | 0x3a...0x5a | 0x5c | 0x5e...0x7e => {
                                let key = self.parse_key(b']', true)?;
                                self.parse_map_value(key, node)?;
                                // self.depth += 1;
                                Ok(true)
                            },
                            c => {
                                Err(de::Error::custom(format!("unexpected character: {}", c)))
                            },
                        }
                    },
                    // This means the key should be a root key
                    // of the form "abc" or "abc[...]"
                    // We do actually allow integer keys here since they cannot
                    // be confused with sequences
                    _ => {
                        println!("Parsing root key: \n{:?}", self);
                        let key = {
                            self.parse_key(b'[', false)?
                        };
                        println!("Parsing map value: \n{:?}", self);
                        self.parse_map_value(key, node)?;
                        // self.depth += 1;
                        Ok(true)
                    },
                }
            },
            // Ran out of characters to parse
            None => Ok(false),
        }
    }
}
