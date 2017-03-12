//! Deserialization support for querystrings.

use serde::de;
#[doc(inline)]
pub use serde::de::value::Error;
use serde::de::value::MapDeserializer;

use std::collections::btree_map::{BTreeMap, Entry, IntoIter};
use std::io::Read;
use url::percent_encoding;

///
pub struct Config {
    max_depth: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { max_depth: 6 }
    }
}

impl Config {
    pub fn max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }
}

impl Config {
    pub fn from_bytes<T: de::Deserialize>(&self,
                                          input: &[u8])
                                          -> Result<T, Error> {
        T::deserialize(Deserializer::with_config(self, input))
    }

    pub fn from_str<T: de::Deserialize>(&self,
                                        input: &str)
                                        -> Result<T, Error> {
        self.from_bytes(input.as_bytes())
    }

    pub fn from_reader<T, R>(&self, mut reader: R) -> Result<T, Error>
        where T: de::Deserialize,
              R: Read,
    {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)
            .map_err(|e| {
                de::Error::custom(format_args!("could not read input: {}", e))
            })?;
        self.from_bytes(&buf)
        // from_bytes(&buf)
        // T::deserialize(Deserializer::with_config(self, input.as_bytes()))
    }
}
/// Deserializes a query-string from a `&[u8]`.
///
/// ```
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_qs;
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Query {
///     name: String,
///     age: u8,
///     occupation: String,
/// }
///
/// # fn main(){
/// let q =  Query {
///     name: "Alice".to_owned(),
///     age: 24,
///     occupation: "Student".to_owned(),
/// };
///
/// assert_eq!(
///     serde_qs::from_bytes::<Query>(
///         "name=Alice&age=24&occupation=Student".as_bytes()
///     ),
///     Ok(q));
/// # }
/// ```
pub fn from_bytes<T: de::Deserialize>(input: &[u8]) -> Result<T, Error> {
    Config::default().from_bytes(input)
}

/// Deserializes a query-string from a `&str`.
///
/// ```
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_qs;
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Query {
///     name: String,
///     age: u8,
///     occupation: String,
/// }
///
/// # fn main(){
/// let q =  Query {
///     name: "Alice".to_owned(),
///     age: 24,
///     occupation: "Student".to_owned(),
/// };
///
/// assert_eq!(
///     serde_qs::from_str::<Query>("name=Alice&age=24&occupation=Student"),
///     Ok(q));
/// # }
/// ```
pub fn from_str<T: de::Deserialize>(input: &str) -> Result<T, Error> {
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T, Error>
    where T: de::Deserialize,
          R: Read,
{
    let mut buf = vec![];
    reader.read_to_end(&mut buf)
        .map_err(|e| {
            de::Error::custom(format_args!("could not read input: {}", e))
        })?;
    from_bytes(&buf)
}

/// A deserializer for the query-string format.
///
/// Supported top-level outputs are structs and maps.
pub struct Deserializer {
    iter: IntoIter<String, Level>,
    value: Option<Level>,
}

#[derive(Debug)]
enum Level {
    Nested(BTreeMap<String, Level>),
    Sequence(Vec<Level>),
    Flat(String),
    Invalid(&'static str),
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

struct Parser<I: Iterator<Item = u8>> {
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
    #[inline]
    fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if !self.acc.is_empty() {
            self.peeked
        } else {
            if let Some(x) = self.inner.next() {
                self.acc.push(x);
                self.peeked = Some(x);
                Some(x)
            } else {
                None
            }
        }
    }

    fn parse_key(&mut self,
                 end_on: u8,
                 consume: bool)
                 -> Result<String, Error> {
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
                       -> Result<(), Error> {
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
                    let value = value.map_err(|_e| de::Error::custom("blah"))?;
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
                    panic!("Unexpected character");
                },
            }
        } else {
            insert_into_map(node, key, "".to_string());
            Ok(())
        }
    }

    fn parse_seq_value(&mut self, node: &mut Level) -> Result<(), Error> {
        match tu!(self.peek()) {
            b'=' => {
                self.acc.clear();
                for b in self.inner.by_ref().take_while(|b| b != &b'&') {
                    self.acc.push(b);
                }
                let value = String::from_utf8(self.acc.split_off(0))
                    .map(|s| s.into());
                let value =
                    value.map_err(|e| de::Error::custom(e.to_string()))?;
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


    fn parse(&mut self, node: &mut Level) -> Result<bool, Error> {
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
                                panic!("");

                            },
                            // key is simply "[]", so treat as a seq.
                            b']' => {
                                self.acc.clear();
                                // println!("Empty key => vector");
                                // println!("{:?}", node);
                                self.parse_seq_value(node)?;
                                self.depth += 1;
                                Ok(true)

                            },
                            // Key is "[a..." so parse up to the closing "]"
                            0x20...0x7e => {
                                let key = self.parse_key(b']', true).unwrap();
                                // key.into()
                                // println!("key: {:?}", key);
                                self.parse_map_value(key.into(), node)?;
                                self.depth += 1;
                                Ok(true)

                            },
                            _ => {
                                panic!("");
                            },
                        }
                    },
                    // This means the key should be a root key
                    // of the form "abc" or "abc[...]"
                    0x20...0x7e => {
                        let key = self.parse_key(b'[', false).unwrap();
                        self.parse_map_value(key.into(), node)?;
                        self.depth += 1;
                        Ok(true)
                    },
                    _ => {
                        panic!("");
                    },
                }
            },
            // Ran out of characters to parse
            None => Ok(false),
        }
    }
}

impl Deserializer {
    fn with_map(map: BTreeMap<String, Level>) -> Self {
        Deserializer {
            iter: map.into_iter(),
            value: None,
        }
    }

    /// Returns a new `Deserializer`.
    fn with_config(config: &Config, input: &[u8]) -> Self {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);

        let decoded = percent_encoding::percent_decode(&input);
        let mut parser = Parser {
            inner: decoded,
            acc: Vec::new(),
            peeked: None,
            depth: config.max_depth,
        };

        while let Ok(x) = parser.parse(&mut root) {
            if !x {
                break;
            }
        }
        let iter = match root {
            Level::Nested(map) => map.into_iter(),
            _ => panic!(""),
        };
        Deserializer {
            iter: iter,
            value: None,
        }
    }
}

impl de::Deserializer for Deserializer {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(MapDeserializer::new(self.iter))
    }
    forward_to_deserialize! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        unit
        option
        bytes
        byte_buf
        unit_struct
        // seq
        seq_fixed_size
        newtype_struct
        tuple_struct
        // struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

use serde::de::value::{SeqDeserializer, ValueDeserializer};

impl de::MapVisitor for Deserializer {
    type Error = Error;


    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: de::DeserializeSeed,
    {

        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            return seed.deserialize(key.into_deserializer()).map(Some);
        };
        Ok(None)

    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: de::DeserializeSeed,
    {
        if let Some(v) = self.value.take() {
            seed.deserialize(v.into_deserializer())
        } else {
            Err(de::Error::custom("Somehow the list was empty after a \
                                   non-empty key was returned"))
        }
    }
}

struct LevelDeserializer(Level);

impl de::Deserializer for LevelDeserializer {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        if let Level::Flat(x) = self.0 {
            x.into_deserializer().deserialize(visitor)
        } else {
            Err(de::Error::custom("cannot deserialize value"))
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        if let Level::Nested(x) = self.0 {
            Deserializer::with_map(x).deserialize_map(visitor)
        } else {
            Err(de::Error::custom(format!("value: {:?} does not appear to \
                                           be a map",
                                          self.0)))
        }
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {

        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        match self.0 {
            Level::Nested(map) => {
                SeqDeserializer::new(map.into_iter().map(|(_k, v)| v))
                    .deserialize(visitor)
            },
            Level::Sequence(x) => {
                SeqDeserializer::new(x.into_iter()).deserialize(visitor)
            },
            Level::Flat(x) => {
                SeqDeserializer::new(vec![x].into_iter()).deserialize(visitor)
            },
            _ => {
                Err(de::Error::custom("value does not appear to be a sequence"))
            },
        }
    }

    fn deserialize_seq_fixed_size<V>(self,
                                     _len: usize,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        match self.0 {
            Level::Flat(x) => {
                if x == "" {
                    visitor.visit_none()
                } else {
                    visitor.visit_some(x.into_deserializer())
                }
            },
            _ => Err(de::Error::custom("value does not appear to be a value")),
        }
    }


    forward_to_deserialize! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        unit
        // option
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

impl ValueDeserializer for Level {
    type Deserializer = LevelDeserializer;
    fn into_deserializer(self) -> Self::Deserializer {
        LevelDeserializer(self)
    }
}
