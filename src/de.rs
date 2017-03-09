//! Deserialization support for the `application/x-www-form-urlencoded` format.

use serde::de;

use fnv::FnvHasher;
// use serde::de::MapVisitor;
use std::iter;
use std::hash::BuildHasherDefault;
// use std::collections::hash_map::{HashMap, Entry, IntoIter};
use std::collections::btree_map::{BTreeMap, Entry, IntoIter};

// use std::collections::BTreeMap;
// type MyHasher = BuildHasherDefault<FnvHasher>;

use std::borrow::Cow;

#[doc(inline)]
pub use serde::de::value::Error;
use serde::de::value::MapDeserializer;
use std::io::Read;
// use url::form_urlencoded::Parse as UrlEncodedParse;
use url::form_urlencoded::parse;
use url::percent_encoding;

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
///     serde_qs::from_bytes::<Query>("name=Alice&age=24&occupation=Student".as_bytes()),
///     Ok(q));
/// # }
/// ```
pub fn from_bytes<T: de::Deserialize>(input: &[u8]) -> Result<T, Error> {
    T::deserialize(Deserializer::new(input))
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
    where T: de::Deserialize, R: Read
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
            // None => return Err(de::Error::custom("query string ended before expected"))
            None => panic!("None found here"),
        }
    )
}

use std::str;
use std::iter::Iterator;

struct Parser<I: Iterator<Item=u8>> {
    inner: I,
    acc: Vec<u8>,
    peeked: Option<u8>,
}

impl<I: Iterator<Item=u8>> Iterator for Parser<I>
{
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<I: Iterator<Item=u8>> Parser<I> {
    fn new(iter: I) -> Self {
        Parser {
            inner: iter,
            acc: Vec::new(),
            peeked: None,
        }
    }

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

    fn parse_string_key(&mut self, end_on: u8, consume: bool) -> Result<String, Error> {
        loop {
            match tu!(self.next()) {
                x if x == end_on  => {
                    let res = String::from_utf8(self.acc.split_off(0));
                    self.acc.clear();

                    // Add this character back to the buffer for peek.
                    if !consume {
                        self.acc.push(x);
                        self.peeked = Some(x);
                    }
                    return res.map_err(|_| de::Error::custom("blah"))
                },
                x @ b'=' => {
                    let res = String::from_utf8(self.acc.split_off(0));
                    self.acc.clear();

                    // Add this character back to the buffer for peek.
                    self.acc.push(x);
                    self.peeked = Some(x);
                    return res.map_err(|_| de::Error::custom("blah"))
                }
                x @ b']' | x @ b'[' => {
                    return Err(de::Error::custom(format!("unexpected character {} in query string, waiting for: {}.", x as char, end_on as char)));
                }
                x @ 0x20 ... 0x7e => {
                    self.acc.push(x);
                },
                _ => {
                    return Err(de::Error::custom("unexpected character in query string."));
                }
            }
        }
    }

    fn parse_int_key(&mut self, end_on: u8) -> Result<String, Error> {
        loop {
            match tu!(self.next()) {
                x if x == end_on  => {
                    let res = String::from_utf8(self.acc.split_off(0)).unwrap();
                    self.acc.clear();
                    return Ok(res);
                },
                x @ b'[' => {
                    return Err(de::Error::custom(format!("unexpected character {} in query string, waiting for: {}.", x as char, end_on as char)));
                }
                x @ b'0' ... b'9' => {
                    self.acc.push(x);
                },
                _ => {
                    return Err(de::Error::custom("unexpected character in query string."));
                }
            }
        }

    }

    fn parse_map_value(&mut self, key: String, node: &mut Level) -> Result<(), Error> {
        match tu!(self.peek()) {
            b'=' => {
                self.acc.clear();
                for b in self.inner.by_ref().take_while(|b| b != &b'&') {
                    self.acc.push(b);
                }
                let value = String::from_utf8(self.acc.split_off(0));
                let value = value.map_err(|_e| de::Error::custom("blah"))?;
                // Reached the end of the key string
                if let Level::Nested(ref mut map) = *node {
                    match map.entry(key) {
                        Entry::Occupied(mut o) => {
                            o.insert(Level::Invalid("Multiple values for one key"));
                        },
                        Entry::Vacant(vm) => {
                            vm.insert(Level::Flat(value));
                        }
                    }
                } else {
                    panic!("");
                }
                Ok(())
            },
            _ => {
                // Ok(())
                if let Level::Nested(ref mut map) = *node {
                    self.parse(
                        map.entry(key).or_insert(Level::Nested(BTreeMap::default()))
                    )?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
        }
    }

    fn parse_seq_value(&mut self, node: &mut Level) -> Result<(), Error> {
        match tu!(self.peek()) {
            b'=' => {
                self.acc.clear();
                // let value = str::from_utf8(input.take_while(|b| *b != &b'&').collect());
                // self.acc.extend_from_slice(&self.take_while(|b| *b != &b'&').collect());
                for b in self.inner.by_ref().take_while(|b| b != &b'&') {
                    self.acc.push(b);
                }
                let value = String::from_utf8(self.acc.split_off(0)).map(|s| s.into());
                let value = value.map_err(|e| de::Error::custom(e.to_string()))?;
                // Reached the end of the key string
                if let Level::Sequence(ref mut seq) = *node {
                    seq.push(Level::Flat(value));
                } else {
                    panic!("");
                }
                Ok(())
            },
            _ => {
                Err(de::Error::custom("non-indexed sequence of structs not supported"))
            }
        }
    }


    fn parse(&mut self, node: &mut Level) -> Result<bool, Error> {
        // First character determines parsing type
        match self.peek() {
            Some(x) => match x {
                b'a' ... b'z' | b'A' ... b'Z' => {
                    let key = self.parse_string_key(b'[', false).unwrap();
                    self.parse_map_value(key.into(), node)?;
                    Ok(true)
                },
                b'[' => {
                    self.acc.clear();
                    // let _ = self.next();
                    match tu!(self.peek()) {
                        b'a' ... b'z' | b'A' ... b'Z' => {
                            let key = self.parse_string_key(b']', true).unwrap();
                            // key.into()
                            self.parse_map_value(key.into(), node)?;
                            Ok(true)

                        },
                        b']' => {
                            self.parse_seq_value(node)?;
                            Ok(true)

                        },
                        b'0' ... b'9' => {
                            let key = self.parse_int_key(b']').unwrap();
                            self.parse_map_value(key, node)?;
                            Ok(true)
                        },
                        _ => {
                            panic!("");
                        }
                    }
                },
                _ => {
                    panic!("");
                }
            },
            // Ran out of characters to parse
            None => return Ok(false)
        }
    }

}

impl Deserializer {
    fn with_map(map: BTreeMap<String,Level>) -> Self {
        Deserializer {
            iter: map.into_iter(),
            value: None,
        }
    }



    /// Returns a new `Deserializer`.
    pub fn new(input: &[u8]) -> Self {
        let map = BTreeMap::default();
        let mut root = Level::Nested(map);

        let decoded = percent_encoding::percent_decode(&input);
        let mut parser = Parser::new(decoded);
        while let Ok(x) = parser.parse(&mut root) {
            if !x {
                break
            }
        }
        // self.input = Some(decoded.as_bytes());
        // println!("{:?}", root);
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

    // _serde::Deserializer::deserialize_struct(deserializer,"A", FIELDS, __Visitor)
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_map(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
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
            return seed.deserialize(key.into_deserializer()).map(Some)
        };
        Ok(None)
    
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: de::DeserializeSeed,
    {
        if let Some(v) = self.value.take() {
            seed.deserialize(v.into_deserializer())
        } else {
            Err(de::Error::custom("Somehow the list was empty after a non-empty key was returned"))
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
            Err(de::Error::custom("value does not appear to be a map"))
        }
    }

    // _serde::Deserializer::deserialize_struct(deserializer,"A", FIELDS, __Visitor)
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {

        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        match self.0 {
            Level::Nested(map) => {
                SeqDeserializer::new(map.into_iter().map(|(_k, v)| v)).deserialize(visitor)
            },
            Level::Sequence(x) => {
                SeqDeserializer::new(x.into_iter()).deserialize(visitor)
            },
            Level::Flat(x) => {
                SeqDeserializer::new(vec!(x).into_iter()).deserialize(visitor)
            }
            _ => {
                Err(de::Error::custom("value does not appear to be a sequence"))
            }
        }
    }

    fn deserialize_seq_fixed_size<V>(self, _len: usize, visitor: V) 
        -> Result<V::Value, Self::Error> where V: de::Visitor
    {
        self.deserialize_seq(visitor)
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
        newtype_struct
        tuple_struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

impl ValueDeserializer for Level 
{
    type Deserializer = LevelDeserializer;
    fn into_deserializer(self) -> Self::Deserializer {
        LevelDeserializer(self)
    }
}
