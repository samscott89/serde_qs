//! Deserialization support for querystrings.

mod parse;

pub use de::parse::Config;

use data_encoding;

use serde::de;
use serde::de::IntoDeserializer;

use url::percent_encoding;

use std::collections::btree_map::{BTreeMap, Entry, IntoIter};
use std::io::{self,Read};
use std::fmt::Display;
use std::string;

error_chain! {
    errors { Custom(msg: String) }
    foreign_links {
        Decoding(data_encoding::decode::Error);
        Io(io::Error);
        Utf8(string::FromUtf8Error);
    }
}

impl Error {
    fn top_level(object: &'static str) -> Self {
        ErrorKind::Custom(format!("cannot deserialize {} at the top level.\
                           Try deserializing into a struct.", object)).into()

    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self 
        where T: Display {
            ErrorKind::Custom(msg.to_string()).into()
    }
}

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
    reader.read_to_end(&mut buf)
        .map_err(|e| {
            ErrorKind::Io(e)
        })?;
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
pub enum Level {
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
    fn with_config(config: &Config, input: &[u8]) -> Self {
        let decoded = percent_encoding::percent_decode(input);
        parse::Parser::new(decoded, vec![], None, config.max_depth()).to_deserializer()

    }
}

impl<'de> de::Deserializer<'de> for QsDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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
        // seq
        // seq_fixed_size
        // newtype_struct
        // tuple_struct
        // struct
        identifier
        // struct_field
        // tuple
        // enum
        ignored_any
    }
}

// use serde::de::IntoDeserializer;
// use serde::de::value::SeqDeserializer;

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
    type Error =  Error;
    type Variant = LevelDeserializer;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        if let Some((key, value)) = self.iter.next() {
            Ok((seed.deserialize(ParsableStringDeserializer(key))?, LevelDeserializer(value)))
        } else {
            Err(de::Error::custom("No more values"))
        }
    }
}

impl<'de> de::EnumAccess<'de> for LevelDeserializer {
    type Error =  Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        match self.0 {
            Level::Flat(x) => {
                Ok((seed.deserialize(ParsableStringDeserializer(x))?, LevelDeserializer(Level::Invalid(""))))
            },
            _ => {
                Err(de::Error::custom("should not be here..."))
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



pub struct LevelDeserializer(Level);

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
    fn to_deserializer(self) -> Result<QsDeserializer> {
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
                // QsDeserializer::with_map(map).deserialize_map(visitor)
                self.deserialize_map(visitor)
            },
            Level::Sequence(_) => {
                self.deserialize_seq(visitor)

            },
            Level::Flat(x) => {
                visitor.visit_string(x)
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
        self.to_deserializer()?.deserialize_map(visitor)

    }

    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.to_deserializer()?.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self, 
        _name: &'static str, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(
        self, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        // self.to_deserializer()?.deserialize_tuple(len, visitor)
        self.deserialize_seq(visitor)

    }
    fn deserialize_tuple_struct<V>(
        self, 
        _name: &'static str, 
        _len: usize, 
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
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
        bytes
        byte_buf
        unit_struct
        // newtype_struct
        // tuple_struct
        identifier
        // tuple
        // enum
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


pub struct ParsableStringDeserializer(String);

impl<'de> de::Deserializer<'de> for ParsableStringDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }


    forward_to_deserialize_any! {
        // bool
        // u8
        // u16
        // u32
        // u64
        // i8
        // i16
        // i32
        // i64
        // f32
        // f64
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
