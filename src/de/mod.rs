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
//! recursively call back to the top level `QsDeserializer` for maps, or when
//! `Level` is a flat value it will attempt to deserialize it to a primitive via
//! `ParsableStringDeserializer`.

mod parse;
mod string_parser;

use crate::error::{Error, Result};

use parse::{ParsedValue, ParsingOptions};
use serde::de;
use string_parser::StringParsingDeserializer;

use crate::map::Map;
use std::borrow::Cow;

/// To override the default serialization parameters, first construct a new
/// Config.
///
/// The `strict` parameter controls whether the deserializer will tolerate
/// encoded brackets as part of the key. For example, serializing the field
/// `a = vec![12]` might give `a[0]=12`. In strict mode, the only string accepted
/// will be this string, whereas in non-strict mode, this can also be deserialized
/// from `a%5B0%5D=12`. Strict mode is more accurate for cases where it a field
/// may contain square brackets.
/// In non-strict mode, the deserializer will generally tolerate unexpected
/// characters.
///
/// A `max_depth` of 0 implies no nesting: the result will be a flat map.
/// This is mostly useful when the maximum nested depth is known beforehand,
/// to prevent denial of service attacks by providing incredibly deeply nested
/// inputs.
///
/// The default value for `max_depth` is 5, and the default mode is `strict=true`.
///
/// ```
/// use serde_qs::Config;
/// use std::collections::HashMap;
///
/// let config = Config::new(0, true);
/// let map: HashMap<String, String> = config.deserialize_str("a[b][c]=1")
///                                          .unwrap();
/// assert_eq!(map.get("a[b][c]").unwrap(), "1");
///
/// let config = Config::new(10, true);
/// let map: HashMap<String, HashMap<String, HashMap<String, String>>> =
///             config.deserialize_str("a[b][c]=1").unwrap();
/// assert_eq!(map.get("a").unwrap().get("b").unwrap().get("c").unwrap(), "1");
/// ```
///
#[derive(Clone, Copy)]
pub struct Config {
    /// Specifies the maximum depth key that `serde_qs` will attempt to
    /// deserialize. Default is 5.
    max_depth: usize,
    /// Strict deserializing mode will not tolerate encoded brackets.
    strict: bool,
}

pub const DEFAULT_CONFIG: Config = Config {
    max_depth: 5,
    strict: true,
};

impl Default for Config {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

impl Config {
    /// Create a new `Config` with the specified `max_depth` and `strict` mode.
    pub fn new(max_depth: usize, strict: bool) -> Self {
        Self { max_depth, strict }
    }
}

impl Config {
    /// Deserializes a querystring from a `&[u8]` using this `Config`.
    pub fn deserialize_bytes<'de, T: de::Deserialize<'de>>(&self, input: &'de [u8]) -> Result<T> {
        T::deserialize(QsDeserializer::with_config(self, input)?)
    }

    /// Deserializes a querystring from a `&str` using this `Config`.
    pub fn deserialize_str<'de, T: de::Deserialize<'de>>(&self, input: &'de str) -> Result<T> {
        self.deserialize_bytes(input.as_bytes())
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
/// Supported top-level outputs are structs and maps.
pub struct QsDeserializer<'a> {
    parsed: parse::ParsedMap<'a>,
}

impl<'a> QsDeserializer<'a> {
    /// Returns a new `QsDeserializer<'a>`.
    pub fn with_config(config: &Config, input: &'a [u8]) -> Result<Self> {
        let parsed = parse::parse(
            input,
            ParsingOptions {
                max_depth: config.max_depth,
                strict: config.strict,
            },
        )?;

        Ok(Self { parsed })
    }

    pub fn new(input: &'a [u8]) -> Result<Self> {
        Self::with_config(&Config::default(), input)
    }

    fn as_nested(&mut self) -> MapDeserializer<'_, 'a> {
        MapDeserializer {
            parsed: &mut self.parsed,
            field_order: None,
            popped_value: None,
        }
    }
}

impl<'de> de::Deserializer<'de> for QsDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.parsed.is_empty() {
            return visitor.visit_unit();
        }

        Err(Error::top_level("primitive"))
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let nested_qs = self.as_nested();
        visitor.visit_map(nested_qs)
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let mut nested_qs = self.as_nested();
        nested_qs.field_order = Some(fields);
        visitor.visit_map(nested_qs)
    }

    /// Throws an error.
    ///
    /// Sequences are not supported at the top level.
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::top_level("sequence"))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    /// Throws an error.
    ///
    /// Tuples are not supported at the top level.
    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
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
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::top_level("tuple struct"))
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self.as_nested())
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
            for (idx, field) in field_order.iter().enumerate() {
                if let Some((key, value)) = crate::map::remove_entry(&mut self.parsed, *field) {
                    *field_order = &field_order[idx + 1..];
                    self.popped_value = Some(value);
                    return seed
                        .deserialize(StringParsingDeserializer::new(key))
                        .map(Some);
                }
            }
        }

        // once we've exhausted the field order, we can
        // just iterate remaining elements in the map
        if let Some((key, value)) = crate::map::pop_first(&mut self.parsed) {
            self.popped_value = Some(value);
            let has_bracket = key.contains('[');
            seed.deserialize(StringParsingDeserializer::new(key))
                .map(Some)
                .map_err(|e| {
                    if has_bracket {
                        de::Error::custom(
                            format!("{e}\nInvalid field contains an encoded bracket -- did you mean to use non-strict mode?\n  https://docs.rs/serde_qs/latest/serde_qs/#strict-vs-non-strict-modes")
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
            seed.deserialize(ValueDeserializer(v))
        } else {
            Err(de::Error::custom(
                "Somehow the map was empty after a non-empty key was returned",
            ))
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
        if let Some((key, value)) = crate::map::pop_first(&mut self.parsed) {
            self.popped_value = Some(value);
            Ok((
                seed.deserialize(StringParsingDeserializer::<'_, Error>::new(key))?,
                self,
            ))
        } else {
            Err(de::Error::custom("No more values"))
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
            seed.deserialize(ValueDeserializer(value))
        } else {
            Err(de::Error::custom("no value to deserialize"))
        }
    }
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.popped_value {
            de::Deserializer::deserialize_seq(ValueDeserializer(value), visitor)
        } else {
            Err(de::Error::custom("no value to deserialize"))
        }
    }
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(value) = self.popped_value {
            de::Deserializer::deserialize_map(ValueDeserializer(value), visitor)
        } else {
            Err(de::Error::custom("no value to deserialize"))
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
            seed.deserialize(ValueDeserializer(v)).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct ValueDeserializer<'a>(ParsedValue<'a>);

macro_rules! forward_to_string_parser {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                if let ParsedValue::String(s) = self.0 {
                    return StringParsingDeserializer::new(s).$meth(visitor);
                } else {
                    return Err(de::Error::custom(
                        format!("expected a string, found {:?}", self.0),
                    ));
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::Map(mut parsed) => visitor.visit_map(MapDeserializer {
                parsed: &mut parsed,
                field_order: None,
                popped_value: None,
            }),
            ParsedValue::Sequence(seq) => visitor.visit_seq(Seq(seq.into_iter())),
            ParsedValue::String(x) => StringParsingDeserializer::new(x).deserialize_any(visitor),
            ParsedValue::Uninitialized => Err(de::Error::custom(
                "internal error: attempted to deserialize unitialised \
                 value",
            )),
            ParsedValue::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::Map(parsed) => {
                #[derive(PartialEq, Eq, PartialOrd, Ord)]
                enum ParsedInteger<'a> {
                    Int(usize),
                    String(Cow<'a, str>),
                }
                // attempt to parse the map as a sequence of ordered keys
                let ordered_map = parsed
                    .into_iter()
                    .map(|(key, v)| match key.parse::<usize>() {
                        Ok(idx) => (ParsedInteger::Int(idx), v),
                        Err(_) => (ParsedInteger::String(key), v),
                    })
                    .collect::<Map<_, _>>();
                visitor.visit_seq(Seq(ordered_map.into_values()))
            }
            ParsedValue::Sequence(seq) => visitor.visit_seq(Seq(seq.into_iter())),
            _ => self.deserialize_any(visitor),
        }
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
        if let ParsedValue::Map(mut parsed) = self.0 {
            visitor.visit_map(MapDeserializer {
                parsed: &mut parsed,
                field_order: Some(fields),
                popped_value: None,
            })
        } else {
            self.deserialize_any(visitor)
        }
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
        if matches!(self.0, ParsedValue::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if matches!(self.0, ParsedValue::Null) {
            visitor.visit_unit()
        } else {
            Err(de::Error::custom("expected unit".to_owned()))
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
            ParsedValue::String(s) => visitor.visit_enum(StringParsingDeserializer::new(s)),
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
        if let ParsedValue::Map(mut parsed) = self.0 {
            visitor.visit_map(MapDeserializer {
                parsed: &mut parsed,
                field_order: None,
                popped_value: None,
            })
        } else {
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::String(s) => match s {
                Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
                Cow::Owned(string) => visitor.visit_string(string),
            },
            ParsedValue::Null => visitor.visit_str(""),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            ParsedValue::String(s) => match s {
                Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
                Cow::Owned(string) => visitor.visit_string(string),
            },
            ParsedValue::Null => visitor.visit_str(""),
            _ => self.deserialize_any(visitor),
        }
    }

    forward_to_deserialize_any! {
        char
        bytes
        byte_buf
        unit_struct
        tuple_struct
        identifier
        tuple
        ignored_any
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
