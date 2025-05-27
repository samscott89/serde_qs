//! Deserialization support for querystrings.
//!
//! ## Design Overview
//!
//! The deserializer uses a two-pass approach to handle arbitrary parameter ordering:
//!
//! 1. **Parse phase**: The querystring is parsed into an intermediate tree structure
//!    (`ParsedValue`) that represents the nested data. This handles bracket notation
//!    and builds the appropriate hierarchy.
//!
//! 2. **Deserialize phase**: The parsed tree is traversed and deserialized into the
//!    target Rust types using serde's visitor pattern.
//!
//! ## Key Components
//!
//! - **`QsDeserializer`**: The top-level deserializer that handles structs and maps.
//!   It can only deserialize map-like structures (key-value pairs).
//!
//! - **`parse` module**: Converts raw querystrings into `ParsedValue` trees,
//!   handling URL decoding and bracket notation parsing.
//!
//! - **`ParsedValueDeserializer`**: Deserializes the intermediate `ParsedValue`
//!   representation into target types. Handles nested maps, sequences, and primitives.
//!
//! ## Example Flow
//!
//! Given `user[name]=John&user[ids][0]=1&user[ids][1]=2`, the parser creates:
//! ```text
//! Map {
//!   "user" => Map {
//!     "name" => String("John"),
//!     "ids" => Sequence [String("1"), String("2")]
//!   }
//! }
//! ```
//!
//! This intermediate structure is then deserialized into the target Rust types.

mod parse;
mod string_parser;

use crate::{
    error::{Error, Result},
    Config,
};

use parse::{Key, ParsedValue};
use serde::de;
use string_parser::StringParsingDeserializer;

use std::{borrow::Cow, iter::FromIterator};

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
pub fn from_bytes<'de, T: de::Deserialize<'de>>(input: &'de [u8]) -> Result<T> {
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
pub fn from_str<'de, T: de::Deserialize<'de>>(input: &'de str) -> Result<T> {
    from_bytes(input.as_bytes())
}

/// A deserializer for the querystring format.
///
/// Primarily useful if needing to use in combination with other
/// libraries, e.g. `serde_path_to_error`.
pub struct QsDeserializer<'a>(ParsedValue<'a>);

impl<'a> QsDeserializer<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self> {
        Self::with_config(Default::default(), input)
    }

    pub fn with_config(config: Config, input: &'a [u8]) -> Result<Self> {
        let parsed = parse::parse(input, config)?;

        Ok(Self(parsed))
    }

    fn seq_length(&self) -> Option<usize> {
        match &self.0 {
            ParsedValue::Sequence(seq) => Some(seq.len()),
            #[cfg(feature = "indexmap")]
            ParsedValue::Map(map) => map
                .iter()
                .map(|(k, _)| match k {
                    Key::Int(i) => Some(1 + *i as usize),
                    Key::String(_) => None,
                })
                .max()
                .unwrap_or(Some(0)),
            #[cfg(not(feature = "indexmap"))]
            ParsedValue::Map(map) => map
                .last_key_value()
                .map(|(k, _)| match k {
                    Key::Int(i) => Some(1 + *i as usize),
                    Key::String(_) => None,
                })
                .unwrap_or(Some(0)),
            ParsedValue::String(_) => Some(1),
            ParsedValue::Null | ParsedValue::NoValue => Some(0),
            ParsedValue::Uninitialized => None,
        }
    }
}

struct MapDeserializer<'a, 'qs: 'a> {
    parsed: &'a mut parse::ParsedMap<'qs>,
    field_order: Option<&'static [&'static str]>,
    popped_value: Option<ParsedValue<'qs>>,
}

impl<'a, 'de: 'a> de::MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        // we'll prefer to use the field order if it exists
        if let Some(field_order) = &mut self.field_order {
            while let Some((field, rem)) = field_order.split_first() {
                *field_order = rem;
                let field_key = (*field).into();
                if let Some(value) = crate::map::remove(self.parsed, &field_key) {
                    self.popped_value = Some(value);
                    return seed
                        .deserialize(StringParsingDeserializer::new_str(field))
                        .map(Some);
                }
            }
        }

        // once we've exhausted the field order, we can
        // just iterate remaining elements in the map
        if let Some((key, value)) = crate::map::pop_first(self.parsed) {
            self.popped_value = Some(value);
            let has_bracket = matches!(key, Key::String(ref s) if s.contains(&b'['));
            key.deserialize_seed(seed)
                .map(Some)
                .map_err(|e| {
                    if has_bracket {
                        Error::custom(
                            format!("{e}\nInvalid field contains an encoded bracket -- consider using form encoding mode\n  https://docs.rs/serde_qs/latest/serde_qs/#query-string-vs-form-encoding")
                            , &self.parsed
                        )
                    } else {
                        e
                    }
                })
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(v) = self.popped_value.take() {
            seed.deserialize(QsDeserializer(v))
        } else {
            Err(Error::custom(
                "Somehow the map was empty after a non-empty key was returned",
                &self.parsed,
            ))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        if let Some(field_order) = self.field_order {
            Some(field_order.len())
        } else {
            Some(self.parsed.len())
        }
    }
}

impl<'a, 'de: 'a> de::EnumAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = crate::map::pop_first(self.parsed) {
            self.popped_value = Some(value);
            Ok((key.deserialize_seed(seed)?, self))
        } else {
            Err(Error::custom("No more values", &self.parsed))
        }
    }
}

impl<'a, 'de: 'a> de::VariantAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.popped_value {
            seed.deserialize(QsDeserializer(value))
        } else {
            Err(Error::custom("no value to deserialize", &self.parsed))
        }
    }
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.popped_value {
            de::Deserializer::deserialize_seq(QsDeserializer(value), visitor)
        } else {
            Err(Error::custom("no value to deserialize", &self.parsed))
        }
    }
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.popped_value {
            de::Deserializer::deserialize_map(QsDeserializer(value), visitor)
        } else {
            Err(Error::custom("no value to deserialize", &self.parsed))
        }
    }
}

struct Seq<'a, I: Iterator<Item = ParsedValue<'a>>>(I);

impl<'de, I: Iterator<Item = ParsedValue<'de>>> de::SeqAccess<'de> for Seq<'de, I> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(v) = self.0.next() {
            seed.deserialize(QsDeserializer(v)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.0.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'a, I> OrderedSeq<'a, I>
where
    I: Iterator<Item = (Key<'a>, ParsedValue<'a>)>,
{
    /// Creates a new `OrderedSeq` from an iterator.
    pub fn new(iter: I) -> Self {
        Self { iter, counter: 0 }
    }
}

struct OrderedSeq<'a, I: Iterator<Item = (Key<'a>, ParsedValue<'a>)>> {
    iter: I,
    counter: u32,
}

impl<'de, I: Iterator<Item = (Key<'de>, ParsedValue<'de>)>> de::SeqAccess<'de>
    for OrderedSeq<'de, I>
{
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.iter.next() {
            match k {
                Key::Int(i) if i == self.counter => {
                    self.counter = self.counter.checked_add(1).ok_or_else(|| {
                        Error::custom("cannot deserialize more than u32::MAX elements", &(k, &v))
                    })?;
                    seed.deserialize(QsDeserializer(v)).map(Some)
                }
                Key::Int(i) => Err(Error::custom(
                    format!("missing index, expected: {} got {i}", self.counter),
                    &(k, v),
                )),
                Key::String(ref bytes) => {
                    let key = std::str::from_utf8(bytes).unwrap_or("<non-utf8>");
                    Err(Error::custom(
                        format!("expected an integer index, found a string key `{key}`"),
                        &(k, v),
                    ))
                }
            }
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        // cannot provide a size hint since we might have gaps
        None
    }
}

fn get_last_string_value<'a>(seq: &mut Vec<ParsedValue<'a>>) -> Result<Cow<'a, [u8]>> {
    let Some(last) = seq.pop() else {
        return Err(Error::custom(
            "internal error: expected a string, found empty sequence",
            &seq,
        ));
    };

    if let ParsedValue::String(s) = last {
        Ok(s)
    } else {
        Err(Error::custom(
            format!("expected a string, found {:?}", last),
            &seq,
        ))
    }
}

macro_rules! forward_to_string_parser {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                let s = match self.0 {
                    ParsedValue::String(ref s) if s.is_empty() => {
                        return Err(Error::custom(
                            format!("expected a `{}`, found an empty string", stringify!($ty)),
                            &self.0
                        ));
                    }
                    ParsedValue::String(s) => {
                        s
                    }
                    ParsedValue::Sequence(mut seq) => {
                        get_last_string_value(&mut seq)?
                    }
                    _ => {
                        return Err(Error::custom(
                            format!("expected a string, found {:?}", self.0),
                            &self.0,
                        ));
                    }
                };
                let deserializer = StringParsingDeserializer::new(s)?;
                return deserializer.$meth(visitor);
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for QsDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::Map(mut parsed) => {
                // scan the map to check if all keys are integers
                // if so, we'll treat it as a sequence
                if parsed.keys().all(|k| matches!(k, Key::Int(_))) {
                    visitor.visit_seq(OrderedSeq::new(parsed.into_iter()))
                } else {
                    visitor.visit_map(MapDeserializer {
                        parsed: &mut parsed,
                        field_order: None,
                        popped_value: None,
                    })
                }
            }
            ParsedValue::Sequence(seq) => visitor.visit_seq(Seq(seq.into_iter())),
            ParsedValue::String(x) => StringParsingDeserializer::new(x)?.deserialize_any(visitor),
            ParsedValue::Uninitialized => Err(Error::custom(
                "internal error: attempted to deserialize unitialised \
                 value",
                &self.0,
            )),

            ParsedValue::Null => {
                StringParsingDeserializer::new(Cow::Borrowed(b""))?.deserialize_any(visitor)
            }
            ParsedValue::NoValue => visitor.visit_unit(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            #[cfg(feature = "indexmap")]
            ParsedValue::Map(mut parsed) => {
                // when using indexmap, we need to first sort the keys
                // or they will be in insertion order
                parsed.sort_unstable_keys();
                visitor.visit_seq(OrderedSeq::new(parsed.into_iter()))
            }
            #[cfg(not(feature = "indexmap"))]
            ParsedValue::Map(parsed) => visitor.visit_seq(OrderedSeq::new(parsed.into_iter())),
            ParsedValue::Sequence(seq) => visitor.visit_seq(Seq(seq.into_iter())),
            // if we have a single string key, but expect a sequence
            // we'll treat it as a sequence of one
            ParsedValue::String(s) => {
                visitor.visit_seq(Seq(std::iter::once(ParsedValue::String(s))))
            }
            ParsedValue::Null | ParsedValue::NoValue => visitor.visit_seq(Seq(std::iter::empty())),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(seq_len) = self.seq_length() {
            if seq_len != len {
                return Err(Error::custom(
                    format!("expected a tuple of length {len}, got length {seq_len}"),
                    &self.0,
                ));
            }
        }
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(seq_len) = self.seq_length() {
            if seq_len != len {
                return Err(Error::custom(
                    format!("expected tuple struct `{name}` of length {len}, got length {seq_len}"),
                    &self.0,
                ));
            }
        }
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut map =
            match self.0 {
                ParsedValue::Map(map) => map,
                ParsedValue::Sequence(parsed_values) => {
                    // if we have a sequence, we can treat it as a map
                    // with the field names as keys
                    if parsed_values.len() != fields.len() {
                        return Err(Error::custom(
                            format!(
                                "expected {fields:?} but got {} elements in sequence",
                                parsed_values.len()
                            ),
                            &parsed_values,
                        ));
                    }

                    parse::ParsedMap::from_iter(fields.iter().zip(parsed_values).map(
                        |(field, value)| (Key::String(Cow::Borrowed(field.as_bytes())), value),
                    ))
                }
                ParsedValue::Null | ParsedValue::NoValue => {
                    // if we have no value, we can just return an empty map
                    parse::ParsedMap::default()
                }
                _ => return self.deserialize_any(visitor),
            };

        visitor.visit_map(MapDeserializer {
            parsed: &mut map,
            field_order: Some(fields),
            popped_value: None,
        })
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::NoValue => visitor.visit_none(),
            ParsedValue::Null => visitor.visit_some(QsDeserializer(ParsedValue::NoValue)),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if matches!(self.0, ParsedValue::NoValue | ParsedValue::Null)
            || matches!(self.0, ParsedValue::String(ref s) if s.is_empty())
        {
            visitor.visit_unit()
        } else {
            Err(Error::custom("expected unit".to_owned(), &self.0))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if matches!(self.0, ParsedValue::NoValue | ParsedValue::Null)
            || matches!(self.0, ParsedValue::String(ref s) if s.is_empty())
        {
            visitor.visit_unit()
        } else {
            Err(Error::custom(
                format!(
                    "expected unit struct `{name}`, got: {}",
                    &self.0.type_name()
                ),
                &self.0,
            ))
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::Map(mut parsed) => visitor.visit_enum(MapDeserializer {
                parsed: &mut parsed,
                field_order: None,
                popped_value: None,
            }),
            ParsedValue::String(s) => visitor.visit_enum(StringParsingDeserializer::new(s)?),
            _ => self.deserialize_any(visitor),
        }
    }

    /// given the hint that this is a map, will first
    /// attempt to deserialize ordered sequences into a map
    /// otherwise, follows the any code path
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::Map(mut parsed) => visitor.visit_map(MapDeserializer {
                parsed: &mut parsed,
                field_order: None,
                popped_value: None,
            }),
            ParsedValue::Null | ParsedValue::NoValue => {
                let mut empty_map = parse::ParsedMap::default();
                visitor.visit_map(MapDeserializer {
                    parsed: &mut empty_map,
                    field_order: None,
                    popped_value: None,
                })
            }
            ParsedValue::String(s) => {
                // if we have a single string key, but expect a map
                // we'll treat it as a map with one key
                let mut parsed = parse::ParsedMap::default();
                parsed.insert(Key::String(Cow::Borrowed(b"")), ParsedValue::String(s));
                visitor.visit_map(MapDeserializer {
                    parsed: &mut parsed,
                    field_order: None,
                    popped_value: None,
                })
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let s = match self.0 {
            ParsedValue::String(s) => s,
            ParsedValue::Sequence(mut seq) => get_last_string_value(&mut seq)?,
            ParsedValue::Null | ParsedValue::NoValue => {
                return visitor.visit_str("");
            }
            _ => return self.deserialize_any(visitor),
        };

        match string_parser::decode_utf8(s)? {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let s = match self.0 {
            ParsedValue::String(s) => s,
            ParsedValue::Sequence(mut seq) => get_last_string_value(&mut seq)?,
            ParsedValue::Null | ParsedValue::NoValue => {
                return visitor.visit_bytes(&[]);
            }
            _ => return self.deserialize_any(visitor),
        };
        match s {
            Cow::Borrowed(s) => visitor.visit_borrowed_bytes(s),
            Cow::Owned(s) => visitor.visit_byte_buf(s),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            // for ignored values, we wont attempt to parse the value
            // as a UTF8 string, but rather just pass the bytes along.
            // since the value is ignored anyway, this is great since
            // we'll just drop it and avoid raising UTF8 errors.
            ParsedValue::String(cow) => match cow {
                Cow::Borrowed(s) => visitor.visit_borrowed_bytes(s),
                Cow::Owned(s) => visitor.visit_byte_buf(s),
            },
            _ => self.deserialize_any(visitor),
        }
    }

    forward_to_deserialize_any! {
        char
        identifier
    }

    forward_to_string_parser! {
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
