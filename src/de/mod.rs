//! Deserialization support for querystrings.

//! ### An overview of the design of `QsDeserializer`
//!
//! This code is designed to handle non-ordered query parameters. For example,
//! `struct { a: Vec<u8>, b: String }` might be serialized as either
//! `a[0]=1&a[1]=2&b=Hello or a[1]=2&b=Hello&a[0]=1`.
//!
//! In order to cover the latter case, we have two options: scan through the
//! string each time we need to find a particular key - worst case O(n^2 )
//! running time; or pre-parse the list into a map structure, and then
//! deserialize the map.
//!
//! We opt for the latter. But a TODO is implement the first case, which could
//! potentially be more desirable, especially when the keys are known to be in
//! order.
//!
//! The `parse` module handles this step of deserializing a querystring into the
//! map structure. This uses `rust_url::percent_encoding` to handle 
//! first converting the string.
//!
//! From here, there are two main `Deserializer` objects: `QsDeserializer` and
//! `LevelDeserializer`.
//!
//! The former is the top-level deserializer which is effectively only capable
//! of deserializing map-like objects (i.e. those with (key, value) pairs).
//! Hence, structs, maps, and enums are supported at this level.
//!
//! Each key is a `String`, and deserialized from a `String`. The values are
//! `Level` elements. This is a recursive structure which can either be a "flat
//! value", i.e. just a string, or a sequence or map of these elements. This can
//! be thought of as similar to the `serde_json::Value` enum.
//!
//! Each `Level` can be deserialized through `LevelDeserializer`. This will
//! recursively call back to the top level `QsDeserializer`, or when `Level` is
//! a flat value it will attempt to deserialize it to a primitive via
//! `ParsableStringDeserializer`.



mod parse;

pub use de::parse::Config;
use error::*;

use data_encoding::base64url as base64;

use serde::de;
use serde::de::IntoDeserializer;

use url::percent_encoding;

use std::collections::btree_map::{BTreeMap, Entry, IntoIter};
use std::io::Read;

/// Deserializes a querystring from a `&[u8]`.
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
///     ).unwrap(), q);
/// # }
/// ```
pub fn from_bytes<'de, T: de::Deserialize<'de>>(input: &[u8]) -> Result<T> {
    Config::default().deserialize_bytes(input)
}

/// Deserializes a querystring from a `&str`.
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
///     serde_qs::from_str::<Query>("name=Alice&age=24&occupation=Student").unwrap(),
///     q);
/// # }
/// ```
pub fn from_str<'de, T: de::Deserialize<'de>>(input: &str) -> Result<T> {
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<'de, T, R>(mut reader: R) -> Result<T>
    where T: de::Deserialize<'de>,
          R: Read,
{
    let mut buf = vec![];
    let _ = reader.read_to_end(&mut buf)
        .map_err(Error::from)?;
    from_bytes(&buf)
}

/// A deserializer for the querystring format.
///
/// Supported top-level outputs are structs and maps.
pub struct QsDeserializer {
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

impl QsDeserializer {
    fn with_map(map: BTreeMap<String, Level>) -> Self {
        QsDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }

    /// Returns a new `QsDeserializer`.
    pub fn with_config(config: &Config, input: &[u8]) -> Self {
        let decoded = percent_encoding::percent_decode(input);
        parse::Parser::new(decoded, vec![], None, config.max_depth()).as_deserializer()

    }
}

impl<'de> de::Deserializer<'de> for QsDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        Err(Error::top_level("primitive"))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    /// Throws an error.
    ///
    /// Sequences are not supported at the top level.
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        Err(Error::top_level("sequence"))
    }

    fn deserialize_newtype_struct<V>(
        self, 
        _name: &'static str, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_map(visitor)
    }

    /// Throws an error.
    ///
    /// Tuples are not supported at the top level.
    fn deserialize_tuple<V>(
        self, 
        _len: usize, 
        _visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        Err(Error::top_level("tuple"))

    }

    /// Throws an error.
    ///
    /// TupleStructs are not supported at the top level.
    fn deserialize_tuple_struct<V>(
        self, 
        _name: &'static str, 
        _len: usize, 
        _visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        Err(Error::top_level("tuple struct"))
    }

    fn deserialize_enum<V>(
        self, 
        _name: &'static str, 
        _variants: &'static [&'static str], 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
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
        identifier
        ignored_any
    }
}

impl<'de> de::MapAccess<'de> for QsDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            return seed.deserialize(ParsableStringDeserializer(key)).map(Some);
        };
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed<'de>,
    {
        if let Some(v) = self.value.take() {
            seed.deserialize(LevelDeserializer(v))
        } else {
            Err(de::Error::custom("Somehow the list was empty after a \
                                   non-empty key was returned"))
        }
    }
}

impl<'de> de::EnumAccess<'de> for QsDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            Ok((seed.deserialize(ParsableStringDeserializer(key))?, self))
        } else {
            Err(de::Error::custom("No more values"))
        }
    }
}

impl<'de> de::VariantAccess<'de> for QsDeserializer {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>
    {
        if let Some(value) = self.value {
            seed.deserialize(LevelDeserializer(value))
        } else {
            Err(de::Error::custom("no value to deserialize"))
        }

    }
    fn tuple_variant<V>(
        self, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        if let Some(value) = self.value {
            de::Deserializer::deserialize_seq(LevelDeserializer(value), visitor)
        } else {
            Err(de::Error::custom("no value to deserialize"))
        }
    }
    fn struct_variant<V>(
        self, 
        _fields: &'static [&'static str], 
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        if let Some(value) = self.value {
            de::Deserializer::deserialize_map(LevelDeserializer(value), visitor)
        } else {
            Err(de::Error::custom("no value to deserialize"))
        }
    }
}

impl<'de> de::EnumAccess<'de> for LevelDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        match self.0 {
            Level::Flat(x) => {
                Ok((seed.deserialize(ParsableStringDeserializer(x))?,
                    LevelDeserializer(Level::Invalid("this value can only deserialize to a UnitVariant"))))
            },
            _ => {
                Err(de::Error::custom("this value can only deserialize to a UnitVariant"))
            }
        }
    }
}

impl<'de> de::VariantAccess<'de> for LevelDeserializer {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>
    {
        seed.deserialize(self)

    }
    fn tuple_variant<V>(
        self, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        de::Deserializer::deserialize_seq(self, visitor)

    }
    fn struct_variant<V>(
        self, 
        _fields: &'static [&'static str], 
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

struct LevelSeq<I: Iterator<Item=Level>>(I);

impl<'de, I: Iterator<Item=Level>> de::SeqAccess<'de> for LevelSeq<I> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed<'de>
    {
        if let Some(v) = self.0.next() {
            seed.deserialize(LevelDeserializer(v)).map(Some)
        } else {
            Ok(None)
        }
    }
}



struct LevelDeserializer(Level);

macro_rules! deserialize_primitive {
    ($ty:ident, $method:ident, $visit_method:ident) => (
        fn $method<V>(self, visitor: V) -> Result<V::Value>
            where V: de::Visitor<'de>,
        {
            match self.0 {
                Level::Nested(_) => {
                    Err(de::Error::custom(format!("Expected: {:?}, got a Map",
                                                  stringify!($ty))))
                },
                Level::Sequence(_) => {
                    Err(de::Error::custom(format!("Expected: {:?}, got a Sequence",
                                                  stringify!($ty))))
                },
                Level::Flat(x) => {
                    ParsableStringDeserializer(x).$method(visitor)
                },
                Level::Invalid(e) => {
                    Err(de::Error::custom(e))
                }
            }
        }
    )
}

impl LevelDeserializer {
    fn into_deserializer(self) -> Result<QsDeserializer> {
        match self.0 {
            Level::Nested(map) => {
                Ok(QsDeserializer::with_map(map))
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
            l => {
                Err(de::Error::custom(format!("could not convert {:?} to QsDeserializer", l)))
            },
        }
    }
}

impl<'de> de::Deserializer<'de> for LevelDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Nested(_) => {
                self.deserialize_map(visitor)
            },
            Level::Sequence(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Flat(x) => {
                ParsableStringDeserializer(x).deserialize_any(visitor)
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Nested(map) => {
                visitor.visit_seq(LevelSeq(map.into_iter().map(|(_k, v)| v)))
            },
            Level::Sequence(x) => {
                visitor.visit_seq(LevelSeq(x.into_iter()))
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            },
            x => {
                visitor.visit_seq(LevelSeq(vec![x].into_iter()))
            },
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        match self.0 {
            Level::Flat(ref x) if x == "" => {
                visitor.visit_none()
            },
            _ => {
                visitor.visit_some(self)
            },
        }
    }

    fn deserialize_enum<V>(
        self, 
        name: &'static str, 
        variants: &'static [&'static str], 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(map) => {
                 QsDeserializer::with_map(map).deserialize_enum(name, variants, visitor)
            },
            Level::Flat(_) => {
                visitor.visit_enum(self)
            },
            _ => {
                Err(de::Error::custom("value does not appear to be a sequence"))
            },
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.into_deserializer()?.deserialize_map(visitor)

    }

    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.into_deserializer()?.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self, 
        _name: &'static str, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(_) => {
                self.deserialize_map(visitor)
            },
            Level::Sequence(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Flat(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }
    }

    fn deserialize_tuple<V>(
        self, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(_) => {
                self.deserialize_map(visitor)
            },
            Level::Sequence(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Flat(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }
    }
    fn deserialize_tuple_struct<V>(
        self, 
        _name: &'static str, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(_) => {
                self.deserialize_map(visitor)
            },
            Level::Sequence(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Flat(_) => {
                self.deserialize_seq(visitor)
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(_) => {
                Err(de::Error::custom("Expected: base64-encoded string, got a Map"))
            },
            Level::Sequence(_) => {
                Err(de::Error::custom("Expected: base64-encoded string, got a Sequence"))
            },
            Level::Flat(x) => {
                visitor.visit_byte_buf(base64::decode_nopad(x.as_bytes())?)   
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        match self.0 {
            Level::Nested(_) => {
                Err(de::Error::custom("Expected: base64-encoded string, got a Map"))
            },
            Level::Sequence(_) => {
                Err(de::Error::custom("Expected: base64-encoded string, got a Sequence"))
            },
            Level::Flat(x) => {
                visitor.visit_byte_buf(base64::decode_nopad(x.as_bytes())?)   
            },
            Level::Invalid(e) => {
                Err(de::Error::custom(e))
            }
        }
    }

    deserialize_primitive!(bool, deserialize_bool, visit_bool);
    deserialize_primitive!(i8,  deserialize_i8, visit_i8);
    deserialize_primitive!(i16, deserialize_i16, visit_i16);
    deserialize_primitive!(i32, deserialize_i32, visit_i32);
    deserialize_primitive!(i64, deserialize_i64, visit_i64);
    deserialize_primitive!(u8,  deserialize_u8, visit_u8);
    deserialize_primitive!(u16, deserialize_u16, visit_u16);
    deserialize_primitive!(u32, deserialize_u32, visit_u32);
    deserialize_primitive!(u64, deserialize_u64, visit_u64);
    deserialize_primitive!(f32, deserialize_f32, visit_f32);
    deserialize_primitive!(f64, deserialize_f64, visit_f64);


    forward_to_deserialize_any! {
        char
        str
        string
        unit
        unit_struct
        identifier
        ignored_any
    }
}


macro_rules! forward_parsable_to_deserialize_any {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$meth(visitor),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        )*
    }
}


struct ParsableStringDeserializer(String);

impl<'de> de::Deserializer<'de> for ParsableStringDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        map
        struct
        seq
        option
        char
        str
        string
        unit
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        identifier
        tuple
        enum
        ignored_any
    }

    forward_parsable_to_deserialize_any! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

}
