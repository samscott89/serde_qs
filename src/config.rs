use std::io::Write;

use serde::de;

use crate::error::Result;
use crate::{Deserializer, Serializer};

/// Configuration for serialization and deserialization behavior.
///
/// The `Config` struct allows you to customize how `serde_qs` handles
/// querystrings, including nesting depth limits and encoding preferences.
///
/// ## Nesting Depth
///
/// The `max_depth` parameter controls how deeply nested structures can be.
/// This is important for preventing denial-of-service attacks from maliciously
/// crafted inputs with excessive nesting. A `max_depth` of 0 means no nesting
/// is allowed (flat key-value pairs only).
///
/// Default value: `max_depth = 5`
///
/// ```
/// use serde_qs::Config;
/// use std::collections::HashMap;
///
/// let config = Config { max_depth: 0, ..Default::default() };
/// let map: HashMap<String, String> = config.deserialize_str("a[b][c]=1")
///                                          .unwrap();
/// assert_eq!(map.get("a[b][c]").unwrap(), "1");
///
/// let config = Config { max_depth: 10, ..Default::default() };
/// let map: HashMap<String, HashMap<String, HashMap<String, String>>> =
///             config.deserialize_str("a[b][c]=1").unwrap();
/// assert_eq!(map.get("a").unwrap().get("b").unwrap().get("c").unwrap(), "1");
/// ```
///
#[derive(Clone, Copy, Debug)]
pub struct Config {
    max_depth: usize,
    use_form_encoding: bool,
    array_format: ArrayFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrayFormat {
    /// Use the `a[0]=1&a[1]=2` format.
    Indexed,
    /// Use the `a[]=1&a[]=2` format.
    EmptyIndexed,
    /// Use the `a=1&a=2` format.
    Unindexed,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub const fn new() -> Self {
        Self {
            max_depth: 5,
            use_form_encoding: cfg!(feature = "default_to_form_encoding"),
            array_format: ArrayFormat::Indexed,
        }
    }

    /// Specifies the maximum depth key that `serde_qs` will attempt to
    /// deserialize. Default is 5.
    pub const fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// By default, `serde_qs` uses query-string encoding, as defined
    /// in [WHATWG](https://url.spec.whatwg.org/#query-percent-encode-set).
    ///
    /// This is a relatively lax encoding scheme, which does not
    /// percent encode many characters (like square brackets).
    ///
    /// This makes it possible to encode nested keys like `a[b][c]=1`
    /// in a relatively compact way. Keys that include square brackets
    /// will get percent-encoded.
    ///
    /// e.g. `{ a: { "[x]": 1 } }` will be encoded as `a[%5Bx%5D]=1`
    ///
    /// Note that when using form encoding this means the keys will get
    /// percent-encoded _twice_.
    ///
    /// e.g. `{ a: { "[x]": 1 } }` will be encoded as `a%5B%255Bx%255D%5D=1`
    ///
    /// To use form encoding, set this to `true`.
    /// Alternatively, you can use the `default_to_form_encoding` Cargo feature
    /// to set this to `true` by default.
    pub const fn use_form_encoding(mut self, use_form_encoding: bool) -> Self {
        self.use_form_encoding = use_form_encoding;
        self
    }

    /// Specifies how arrays should be formatted in the querystring
    /// during serialization.
    ///
    /// The default is `Indexed`, which results in keys like `a[0]=1&a[1]=2`.
    pub const fn array_format(mut self, array_format: ArrayFormat) -> Self {
        self.array_format = array_format;
        self
    }

    /// Deserializes a querystring from a `&[u8]` using this `Config`.
    pub fn deserialize_bytes<'de, T: de::Deserialize<'de>>(self, input: &'de [u8]) -> Result<T> {
        T::deserialize(Deserializer::with_config(self, input)?)
    }

    /// Deserializes a querystring from a `&str` using this `Config`.
    pub fn deserialize_str<'de, T: de::Deserialize<'de>>(self, input: &'de str) -> Result<T> {
        self.deserialize_bytes(input.as_bytes())
    }

    /// Serializes an object to a querystring using this `Config`.
    pub fn serialize_string<T: serde::Serialize>(self, input: &T) -> Result<String> {
        // initialize the buffer with 128 bytes
        // this is a guess based on what `serde_json` does
        let mut buffer = Vec::with_capacity(128);
        let mut serializer = Serializer::new(&mut buffer, self);
        input.serialize(&mut serializer)?;
        String::from_utf8(buffer).map_err(crate::Error::from)
    }

    /// Serializes an object to a querystring using this `Config`.
    pub fn serialize_to_writer<T: serde::Serialize, W: Write>(
        self,
        input: &T,
        writer: &mut W,
    ) -> Result<()> {
        let mut serializer = Serializer::new(writer, self);
        input.serialize(&mut serializer)
    }
}
