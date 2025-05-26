//! Serialization support for querystrings.

mod encode;

use encode::encode;

use serde::ser;

use crate::error::*;

use std::fmt::Display;
use std::io::Write;
use std::str;

/// Serializes a value into a querystring.
///
/// ```
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_qs;
/// #[derive(Deserialize, Serialize)]
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
///
/// assert_eq!(
///     serde_qs::to_string(&q).unwrap(),
///     "name=Alice&age=24&occupation=Student");
/// # }
/// ```
pub fn to_string<T: ser::Serialize>(input: &T) -> Result<String> {
    let config = crate::Config::default();
    config.serialize_string(input)
}

/// Serializes a value into a generic writer object.
///
/// ```
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_qs;
/// #[derive(Deserialize, Serialize)]
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
/// let mut buffer = Vec::new();
/// serde_qs::to_writer(&q, &mut buffer).unwrap();
/// assert_eq!(
///     String::from_utf8(buffer).unwrap(),
///     "name=Alice&age=24&occupation=Student");
/// # }
/// ```
pub fn to_writer<T: ser::Serialize, W: Write>(input: &T, writer: &mut W) -> Result<()> {
    let config = crate::Config::default();
    config.serialize_to_writer(input, writer)
}

/// A serializer for the querystring format.
///
/// This serializer converts Rust data structures into URL-encoded querystrings
/// with support for nested structures using bracket notation.
///
/// ## Key Features
///
/// * **Nested structures**: Serializes nested objects as `parent[child]=value`
/// * **Arrays**: Serializes sequences with indices like `items[0]=a&items[1]=b`
/// * **Type support**: Handles primitives, strings, structs, maps, and sequences
/// * **Encoding modes**: Supports both query-string and form encoding
///
/// ## Implementation Details
///
/// The serializer maintains a key stack to build nested paths. For example,
/// when serializing `{user: {name: "John"}}`, it pushes "user" onto the stack,
/// then serializes "name" as `user[name]=John`.
pub struct QsSerializer<W: Write> {
    writer: W,
    first_kv: bool,
    key: Vec<Vec<u8>>,
    config: crate::Config,
}

impl<W: Write> QsSerializer<W> {
    /// Creates a new `QsSerializer` with the given writer.
    pub fn new(writer: W, config: crate::Config) -> Self {
        Self {
            writer,
            first_kv: true,
            key: Vec::with_capacity(4),
            config,
        }
    }
}

impl<W: Write> QsSerializer<W> {
    /// Pushes a new key segment onto the key stack for nested structures.
    ///
    /// This method builds the bracket notation for nested keys. For example:
    /// - First key "user" becomes: `user`
    /// - Second key "name" becomes: `user[name]`
    /// - Third key "first" becomes: `user[name][first]`
    fn push_key(&mut self, newkey: &[u8]) -> Result<()> {
        let first_key_segment = self.key.is_empty();

        // estimate the required capacity based on
        // the key length and encoding
        // note that if we do require percent-encoding
        // the key, then we'll probably need to grow
        // the capacity -- we're being optimistic here
        // that the common case does not need to encode
        let estimated_capacity = newkey.len()
            + if first_key_segment { 0 } else { 2 }
            + if self.config.use_form_encoding { 6 } else { 2 };
        let mut segment = Vec::with_capacity(estimated_capacity);
        if !first_key_segment {
            if self.config.use_form_encoding {
                segment.extend_from_slice(b"%5B");
            } else {
                segment.push(b'[');
            }
        }

        if newkey
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'-' || *b == b'_' || *b == b'.')
        {
            // optimization for the case where the key
            // is alphanumeric or a few special characters
            // this avoids the percent-encoding overhead
            segment.extend_from_slice(newkey);
        } else {
            for encoded in encode(newkey, self.config.use_form_encoding) {
                segment.extend_from_slice(&encoded);
            }
        }

        if !first_key_segment {
            if self.config.use_form_encoding {
                segment.extend_from_slice(b"%5D");
            } else {
                segment.push(b']');
            }
        }
        self.key.push(segment);
        Ok(())
    }

    /// Writes a key directly to output without adding to the stack.
    ///
    /// This is an optimization for leaf values where we know there won't be
    /// any further nesting. It avoids the allocation of pushing to the key stack
    /// and immediately writes the full key path to the output.
    fn write_key(&mut self, newkey: &[u8]) -> Result<()> {
        if self.key.is_empty() {
            if self.first_kv {
                self.first_kv = false;
            } else {
                self.writer.write_all(b"&")?;
            }
            for encoded in encode(newkey, self.config.use_form_encoding) {
                self.writer.write_all(&encoded)?;
            }
        } else {
            self.write_key_stack()?;
            if self.config.use_form_encoding {
                self.writer.write_all(b"%5B")?;
            } else {
                self.writer.write_all(b"[")?;
            }
            for encoded in encode(newkey, self.config.use_form_encoding) {
                self.writer.write_all(&encoded)?;
            }
            if self.config.use_form_encoding {
                self.writer.write_all(b"%5D")?;
            } else {
                self.writer.write_all(b"]")?;
            }
        }
        Ok(())
    }

    fn write_key_stack(&mut self) -> Result<()> {
        if self.key.is_empty() {
            // this is the case when we used `write_key_out`
            // to write the key without pushing it to the stack
            return Ok(());
        }
        if self.first_kv {
            self.first_kv = false;
        } else {
            self.writer.write_all(b"&")?;
        }
        let Some(first_segment) = self.key.first() else {
            return Err(Error::Custom("internal error: no key found".to_string()));
        };

        // write the key segments
        self.writer.write_all(first_segment)?;
        for segment in self.key.iter().skip(1) {
            self.writer.write_all(segment)?;
        }

        Ok(())
    }

    fn pop_key(&mut self) -> Result<()> {
        let popped = self.key.pop();
        if popped.is_none() {
            return Err(Error::Custom("internal error: no key found".to_string()));
        }
        Ok(())
    }

    fn write_value(&mut self, value: &[u8]) -> Result<()> {
        self.write_key_stack()?;
        self.writer.write_all(b"=")?;
        for encoded in encode(value, self.config.use_form_encoding) {
            self.writer.write_all(&encoded)?;
        }
        Ok(())
    }

    fn write_unit(&mut self) -> Result<()> {
        self.write_key_stack()?;
        self.writer.write_all(b"=")?;
        Ok(())
    }

    fn write_no_value(&mut self) -> Result<()> {
        self.write_key_stack()?;
        Ok(())
    }
}

macro_rules! serialize_itoa {
    (
        $($ty:ty => $meth:ident,)*) => {
        $(
            #[allow(unused_mut)]
            fn $meth(mut self, v: $ty) -> Result<Self::Ok> {
                let mut buffer = itoa::Buffer::new();
                let key = buffer.format(v);
                self.write_value(key.as_bytes())?;
                Ok(())
            }
        )*
    };
}

macro_rules! serialize_ryu {
    (
        $($ty:ty => $meth:ident,)*) => {
        $(
            #[allow(unused_mut)]
            fn $meth(mut self, v: $ty) -> Result<Self::Ok> {
                let mut buffer = ryu::Buffer::new();
                let key = buffer.format(v);
                self.write_value(key.as_bytes())?;
                Ok(())
            }
        )*
    };
}

impl<'a, W: Write> ser::Serializer for &'a mut QsSerializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = QsSeq<'a, W>;
    type SerializeTuple = QsSeq<'a, W>;
    type SerializeTupleStruct = QsSeq<'a, W>;
    type SerializeTupleVariant = QsSeq<'a, W>;
    type SerializeMap = QsMap<'a, W>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    serialize_itoa! {
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
    }
    serialize_ryu! {
        f32 => serialize_f32,
        f64 => serialize_f64,
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        self.write_value(value)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.write_unit()
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        self.write_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.write_key(variant.as_bytes())?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.push_key(variant.as_bytes())?;
        value.serialize(&mut *self)?;
        self.pop_key()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.write_no_value()?;
        Ok(())
    }

    fn serialize_some<T: ?Sized + ser::Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(QsSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(QsSeq::new(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(QsSeq::new(self))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.push_key(variant.as_bytes())?;
        Ok(QsSeq::new(self))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(QsMap::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.push_key(variant.as_bytes())?;
        Ok(self)
    }

    fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        let key = if v {
            b"true" as &'static [u8]
        } else {
            b"false"
        };
        self.write_value(key)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
        let mut b = [0; 4];
        let key = v.encode_utf8(&mut b);
        self.write_value(key.as_bytes())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.write_value(v.as_bytes())?;
        Ok(())
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

#[doc(hidden)]
pub struct QsSeq<'s, W: Write> {
    qs: &'s mut QsSerializer<W>,
    counter: usize,
}

impl<'a, W: Write> QsSeq<'a, W> {
    fn new(qs: &'a mut QsSerializer<W>) -> Self {
        Self { qs, counter: 0 }
    }

    /// Pushes the key to the serializer.
    fn push_key(&mut self) -> Result<()> {
        match self.qs.config.array_format {
            crate::ArrayFormat::Indexed => {
                // indexed arrays have keys like `[0]`, `[1]`, etc.
                let mut buffer = itoa::Buffer::new();
                // encode the next integer key
                let key = buffer.format(self.counter);
                self.qs.push_key(key.as_bytes())?;
            }
            crate::ArrayFormat::EmptyIndexed => {
                // empty indexed arrays have keys like `[]`
                self.qs.push_key(b"")?;
            }
            crate::ArrayFormat::Unindexed => {
                // unindexed arrays have no keys, so nothing to push
            }
        }

        // increment the key
        self.counter += 1;
        Ok(())
    }

    fn pop_key(&mut self) -> Result<()> {
        // pop the key from the serializer (if we pushed one)
        if matches!(
            self.qs.config.array_format,
            crate::ArrayFormat::Indexed | crate::ArrayFormat::EmptyIndexed
        ) {
            self.qs.pop_key()?;
        }
        Ok(())
    }
}

impl<W: Write> ser::SerializeTuple for QsSeq<'_, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key()?;
        value.serialize(&mut *self.qs)?;
        self.pop_key()
    }

    fn end(self) -> Result<Self::Ok> {
        // if we didn't serialize any elements, we'll write a null
        // value
        if self.counter == 0 {
            self.qs.write_unit()?;
        }
        Ok(())
    }
}

impl<W: Write> ser::SerializeSeq for QsSeq<'_, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key()?;
        value.serialize(&mut *self.qs)?;
        self.pop_key()
    }
    fn end(self) -> Result<Self::Ok> {
        // if we didn't serialize any elements, we'll write a null
        // value
        if self.counter == 0 {
            self.qs.write_unit()?;
        }
        Ok(())
    }
}

impl<W: Write> ser::SerializeTupleVariant for QsSeq<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key()?;
        value.serialize(&mut *self.qs)?;
        self.pop_key()
    }

    fn end(self) -> Result<Self::Ok> {
        // after serializing a tuple variant, we need to pop the
        // variant key
        self.qs.pop_key()?;
        Ok(())
    }
}

impl<W: Write> ser::SerializeTupleStruct for QsSeq<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key()?;
        value.serialize(&mut *self.qs)?;
        self.pop_key()
    }

    fn end(self) -> Result<Self::Ok> {
        // if we didn't serialize any elements, we'll write a null
        // value
        if self.counter == 0 {
            self.qs.write_unit()?;
        }
        Ok(())
    }
}

impl<W: Write> ser::SerializeStruct for &mut QsSerializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key(key.as_bytes())?;
        value.serialize(&mut **self)?;
        self.pop_key()
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeStructVariant for &mut QsSerializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.push_key(key.as_bytes())?;
        value.serialize(&mut **self)?;
        self.pop_key()
    }

    fn end(self) -> Result<Self::Ok> {
        self.pop_key()?;
        Ok(())
    }
}

#[doc(hidden)]
pub struct QsMap<'s, W: Write> {
    qs: &'s mut QsSerializer<W>,
    empty: bool,
}

impl<'a, W: Write> QsMap<'a, W> {
    fn new(qs: &'a mut QsSerializer<W>) -> Self {
        Self { qs, empty: true }
    }
}

impl<W: Write> ser::SerializeMap for QsMap<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.empty = false;
        key.serialize(KeySerializer::new(self.qs))?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.qs)?;
        self.qs.pop_key()
    }

    fn end(self) -> Result<Self::Ok> {
        if self.empty {
            // if we didn't serialize any elements, we'll write a null
            // value
            self.qs.write_unit()?;
        }
        Ok(())
    }
}

macro_rules! serialize_key_itoa {
    (
        $($ty:ty => $meth:ident,)*) => {
        $(
            #[allow(unused_mut)]
            fn $meth(mut self, v: $ty) -> Result<Self::Ok> {
                let mut buffer = itoa::Buffer::new();
                let key = buffer.format(v);
                self.push_key(key.as_bytes())?;
                Ok(())
            }
        )*
    };
}

macro_rules! serialize_key_ryu {
    (
        $($ty:ty => $meth:ident,)*) => {
        $(
            #[allow(unused_mut)]
            fn $meth(mut self, v: $ty) -> Result<Self::Ok> {
                let mut buffer = ryu::Buffer::new();
                let key = buffer.format(v);
                self.push_key(key.as_bytes())?;
                Ok(())
            }
        )*
    };
}

struct KeySerializer<'a, W: Write> {
    qs: &'a mut QsSerializer<W>,
}

impl<'a, W: Write> KeySerializer<'a, W> {
    fn new(qs: &'a mut QsSerializer<W>) -> Self {
        Self { qs }
    }

    fn push_key(&mut self, key: &[u8]) -> Result<()> {
        self.qs.push_key(key)?;
        Ok(())
    }
}

impl<W: Write> ser::Serializer for KeySerializer<'_, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    serialize_key_itoa! {
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
    }
    serialize_key_ryu! {
        f32 => serialize_f32,
        f64 => serialize_f64,
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        self.qs.push_key(value)?;
        Ok(())
    }

    /// Returns an error.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.qs.push_key(variant.as_bytes())?;
        Ok(())
    }

    /// Returns an error.
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Unsupported)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Unsupported)
    }

    /// Returns an error.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Unsupported)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Unsupported)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::Unsupported)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Unsupported)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let key = if v {
            b"true" as &'static [u8]
        } else {
            b"false"
        };
        self.qs.push_key(key)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
        let mut b = [0; 4];
        let key = v.encode_utf8(&mut b);
        self.qs.push_key(key.as_bytes())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.qs.push_key(v.as_bytes())?;
        Ok(())
    }
}
