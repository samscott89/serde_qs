use serde::de;

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
    pub fn with_max_depth(depth: usize) -> Config {
        Config {
            max_depth: depth
        }
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Config {
    pub fn deserialize_bytes<'de, T: de::Deserialize<'de>>(&self,
                                                 input: &[u8])
                                                 -> Result<T> {
        T::deserialize(QsDeserializer::with_config(self, input))
    }

    pub fn deserialize_str<'de, T: de::Deserialize<'de>>(&self,
                                               input: &str)
                                               -> Result<T> {
        self.deserialize_bytes(input.as_bytes())
    }

    pub fn deserialize_reader<'de, T, R>(&self, mut reader: R) -> Result<T>
        where T: de::Deserialize<'de>,
              R: Read,
    {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)
            .map_err(|e| {
                ErrorKind::Io(e)
            })?;
        self.deserialize_bytes(&buf)
    }
}


macro_rules! tu {
    ($x:expr) => (
        match $x {
            Some(x) => x,
            None => return Err(
                de::Error::custom("query string ended before expected"))
        }
    )
}

use std::iter::Iterator;
use std::str;

pub struct Parser<I: Iterator<Item = u8>> {
    inner: I,
    acc: Vec<u8>,
    peeked: Option<u8>,
    depth: usize,
}

impl<I: Iterator<Item = u8>> Iterator for Parser<I> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}


fn insert_into_map(node: &mut Level, key: String, value: String) {
    if let Level::Nested(ref mut map) = *node {
        match map.entry(key) {
            Entry::Occupied(mut o) => {
                o.insert(Level::Invalid("Multiple values for one key"));
            },
            Entry::Vacant(vm) => {
                vm.insert(Level::Flat(value));
            },
        }
    } else {
        let mut map = BTreeMap::default();
        map.insert(key, Level::Flat(value));
        *node = Level::Nested(map);
    }
}

impl<I: Iterator<Item = u8>> Parser<I> {
    pub fn new(inner: I, acc: Vec<u8>, peeked: Option<u8>, depth: usize) -> Self {
        Parser {
            inner, acc, peeked, depth
        }
    }

    pub fn as_deserializer(&mut self) -> QsDeserializer {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);
        while let Ok(x) = self.parse(&mut root) {
            if !x {
                break;
            }
        }
        let iter = match root {
            Level::Nested(map) => map.into_iter(),
            _ => panic!("root node should never be able to converted to anything else. Something went seriously wrong."),
        };
        QsDeserializer {
            iter: iter,
            value: None,
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if !self.acc.is_empty() {
            self.peeked
        } else if let Some(x) = self.inner.next() {
            self.acc.push(x);
            self.peeked = Some(x);
            Some(x)
        } else {
            None
        }
    }

    fn parse_key(&mut self,
                 end_on: u8,
                 consume: bool)
                 -> Result<String> {
        loop {
            if let Some(x) = self.next() {
                match x {
                    x if x == end_on => {
                        let res = String::from_utf8(self.acc.split_off(0));
                        self.acc.clear();

                        // Add this character back to the buffer for peek.
                        if !consume {
                            self.acc.push(x);
                            self.peeked = Some(x);
                        }
                        return res.map_err(|_| de::Error::custom("blah"));
                    },
                    b'=' => {
                        // Allow the '=' byte when parsing keys within []
                        if end_on == b']' {
                            self.acc.push(b'=');
                        } else {
                            let res = String::from_utf8(self.acc.split_off(0));
                            self.acc.clear();

                            // Add this character back to the buffer for peek.
                            self.acc.push(b'=');
                            self.peeked = Some(b'=');

                            return res.map_err(|_| de::Error::custom("blah"));
                        }
                    },
                    b' ' => {
                        self.acc.push(b' ');
                    },
                    b'&' => {
                        let res = String::from_utf8(self.acc.split_off(0));
                        self.acc.clear();
                        self.acc.push(b'&');
                        self.peeked = Some(b'&');
                        return res.map_err(|_| de::Error::custom("blah"));
                    },
                    x @ 0x20...0x7e => {
                        self.acc.push(x);
                    },
                    _ => {
                        return Err(de::Error::custom("unexpected character \
                                                      in query string."));
                    },
                }
            } else {
                // End of string.
                let res = String::from_utf8(self.acc.split_off(0));
                self.acc.clear();
                return res.map_err(|_| de::Error::custom("blah"));
            }
        }
    }

    fn parse_map_value(&mut self,
                       key: String,
                       node: &mut Level)
                       -> Result<()> {
        if let Some(x) = self.peek() {
            match x {
                b'=' => {
                    self.acc.clear();
                    for b in self.inner.by_ref().take_while(|b| b != &b'&') {
                        if b == b'+' {
                            self.acc.push(b' ');
                        } else {
                            self.acc.push(b);
                        }
                    }
                    let value = String::from_utf8(self.acc.split_off(0));
                    let value = value.map_err(Error::from)?;
                    // Reached the end of the key string
                    insert_into_map(node, key, value);
                    Ok(())
                },
                b'&' => {
                    insert_into_map(node, key, "".to_string());
                    Ok(())
                },
                b'[' => {
                    if let Level::Invalid(_) = *node {
                        *node = Level::Nested(BTreeMap::default());
                    }
                    if let Level::Nested(ref mut map) = *node {
                        self.depth -= 1;
                        self.parse(map.entry(key)
                                .or_insert(Level::Invalid("uninitialised")))?;
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
            insert_into_map(node, key, "".to_string());
            Ok(())
        }
    }

    fn parse_seq_value(&mut self, node: &mut Level) -> Result<()> {
        match tu!(self.peek()) {
            b'=' => {
                self.acc.clear();
                for b in self.inner.by_ref().take_while(|b| b != &b'&') {
                    self.acc.push(b);
                }
                let value = String::from_utf8(self.acc.split_off(0))
                    .map(|s| s.into());
                let value = value.map_err(Error::from)?;
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
        }
    }


    fn parse(&mut self, node: &mut Level) -> Result<bool> {
        // First character determines parsing type
        if self.depth == 0 {
            let key = self.parse_key(b'\x00', true)?;
            self.parse_map_value(key.into(), node)?;
            self.depth += 1;
            return Ok(true);
        }
        match self.peek() {
            Some(x) => {
                match x {
                    b'[' => {
                        self.acc.clear();
                        // let _ = self.next();
                        match tu!(self.peek()) {
                            // key is of the form "[...", not really allowed.
                            b'[' => {
                                Err(de::Error::custom("found another opening bracket before the closed bracket"))

                            },
                            // key is simply "[]", so treat as a seq.
                            b']' => {
                                self.acc.clear();
                                self.parse_seq_value(node)?;
                                self.depth += 1;
                                Ok(true)

                            },
                            // Key is "[a..." so parse up to the closing "]"
                            0x20...0x5a | 0x5c | 0x5e...0x7e => {
                                let key = self.parse_key(b']', true)?;
                                self.parse_map_value(key.into(), node)?;
                                self.depth += 1;
                                Ok(true)

                            },
                            c => {
                                Err(de::Error::custom(format!("unexpected character: {}", c)))
                            },
                        }
                    },
                    // This means the key should be a root key
                    // of the form "abc" or "abc[...]"
                    0x20...0x5a | 0x5c...0x7e => {
                        let key = self.parse_key(b'[', false)?;
                        self.parse_map_value(key.into(), node)?;
                        self.depth += 1;
                        Ok(true)
                    },
                    c => {
                        Err(de::Error::custom(format!("unexpected character: {}", c)))
                    }
                }
            },
            // Ran out of characters to parse
            None => Ok(false),
        }
    }
}
