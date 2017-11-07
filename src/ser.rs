
//! Serialization support for querystrings.

use data_encoding::BASE64URL_NOPAD as BASE64;
use serde::ser;
use url::form_urlencoded::Serializer as UrlEncodedSerializer;
use url::form_urlencoded::Target as UrlEncodedTarget;

use std::fmt::Display;
use std::borrow::Cow;
use std::str;

use error::*;

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
    let mut urlencoder = UrlEncodedSerializer::new("".to_owned());
    input.serialize(&mut QsSerializer { key: None, urlencoder: &mut urlencoder })?;
    Ok(urlencoder.finish())
}

/// A serializer for the querystring format.
///
/// * Supported top-level inputs are structs and maps.
///
/// * Supported values are currently most primitive types, structs, maps and
///   sequences. Sequences are serialized with an incrementing key index.
///
/// * Newtype structs defer to their inner values.
pub struct QsSerializer<'a, Target: 'a + UrlEncodedTarget> {
    key: Option<Cow<'static, str>>,
    urlencoder: &'a mut UrlEncodedSerializer<Target>,
}

impl<'a, Target: 'a + UrlEncodedTarget> QsSerializer<'a, Target> {
    fn extend_key(&mut self, newkey: &str) {
        let key = if let Some(ref key) = self.key {
            format!("{}[{}]", key, newkey).into()
        } else {
            newkey.to_owned().into()
        };
        self.key = Some(key)
    }

    fn write_value(&mut self, value: &str) -> Result<()> {
        if let Some(ref key) = self.key {
            // returns &Self back anyway
            let _ = self.urlencoder.append_pair(key, value);
            Ok(())
        } else {
            Err(Error::no_key())
        }
    }
}

impl Error {
    fn no_key() -> Self {
        let msg = "tried to serialize a value before serializing key";
        msg.into()
    }
}

macro_rules! serialize_as_string {
    (Qs $($ty:ty => $meth:ident,)*) => {
        $(
            fn $meth(self, v: $ty) -> Result<Self::Ok> {
                self.write_value(&v.to_string())
            }
        )*
    };
    ($($ty:ty => $meth:ident,)*) => {
        $(
            fn $meth(self, v: $ty) -> Result<Self::Ok> {
                Ok(v.to_string())
            }
        )*
    };
}

impl<'a, Target: 'a + UrlEncodedTarget> ser::Serializer for &'a mut QsSerializer<'a, Target> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = QsSeq<'a, Target>;
    type SerializeTuple = QsSeq<'a, Target>;
    type SerializeTupleStruct = QsSeq<'a, Target>;
    type SerializeTupleVariant = QsSeq<'a, Target>;
    type SerializeMap = QsMap<'a, Target>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    serialize_as_string!{
        Qs
        bool => serialize_bool,
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        self.write_value(&BASE64.encode(value))
    }


    fn serialize_unit(self) -> Result<Self::Ok> {
        self.write_value("")
    }

    /// Returns an error.
    fn serialize_unit_struct(self,
                             name: &'static str)
                             -> Result<Self::Ok> {
        self.write_value(name)
    }

    /// Returns an error.
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              variant: &'static str)
                              -> Result<Self::Ok> {
        self.write_value(variant)
    }

    /// Returns an error.
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>
        (self,
         _name: &'static str,
         value: &T)
         -> Result<Self::Ok> {
        value.serialize(self)
    }

    /// Returns an error.
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>
        (self,
         _name: &'static str,
         _variant_index: u32,
         variant: &'static str,
         value: &T)
         -> Result<Self::Ok> {
        self.extend_key(variant);
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + ser::Serialize>
        (self,
         value: &T)
         -> Result<Self::Ok> {
        // Err(ErrorKind::Unsupported.into())
        value.serialize(self)
    }

    /// Returns an error.
    fn serialize_seq(self,
                     _len: Option<usize>)
                     -> Result<Self::SerializeSeq> {
        Ok(QsSeq(self, 0))
    }


    fn serialize_tuple(self,
                       _len: usize)
                       -> Result<Self::SerializeTuple> {
        Ok(QsSeq(self, 0))
    }

    /// Returns an error.
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Ok(QsSeq(self, 0))
    }

    fn serialize_tuple_variant
        (self,
         _name: &'static str,
         _variant_index: u32,
         variant: &'static str,
         _len: usize)
         -> Result<Self::SerializeTupleVariant>
    {
        // self.write(variant)?;
        self.extend_key(variant);
        Ok(QsSeq(self, 0))
    }

    fn serialize_map(self,
                     _len: Option<usize>)
                     -> Result<Self::SerializeMap> {
        Ok(QsMap(self, None))
    }

    fn serialize_struct(self,
                        _name: &'static str,
                        _len: usize)
                        -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant
        (self,
         _name: &'static str,
         _variant_index: u32,
         variant: &'static str,
         _len: usize)
         -> Result<Self::SerializeStructVariant> 
    {
        self.extend_key(variant);
        Ok(self)
    }
}


impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self 
        where T: Display {
            ErrorKind::Custom(msg.to_string()).into()
    }
}

pub struct QsSeq<'a, Target: 'a + UrlEncodedTarget>(&'a mut QsSerializer<'a, Target>, usize);
pub struct QsMap<'a, Target: 'a + UrlEncodedTarget>(&'a mut QsSerializer<'a, Target>, Option<Cow<'a, str>>);


impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeTuple for QsSeq<'a, Target> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        serializer.extend_key(&self.1.to_string());
        self.1 += 1;
        value.serialize(&mut serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())

    }
}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeSeq for QsSeq<'a, Target> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        serializer.extend_key(&self.1.to_string());
        self.1 += 1;
        value.serialize(&mut serializer)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())

    }
}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeStruct for &'a mut QsSerializer<'a, Target>  {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.key.clone(), urlencoder: self.urlencoder };
        serializer.extend_key(key);
        value.serialize(&mut serializer)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeStructVariant for &'a mut QsSerializer<'a, Target> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.key.clone(), urlencoder: self.urlencoder };
        serializer.extend_key(key);
        value.serialize(&mut serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeTupleVariant for QsSeq<'a, Target> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        serializer.extend_key(&self.1.to_string());
        self.1 += 1;
        value.serialize(&mut serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeTupleStruct for QsSeq<'a, Target> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        serializer.extend_key(&self.1.to_string());
        self.1 += 1;
        value.serialize(&mut serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

}

impl<'a, Target: 'a + UrlEncodedTarget> ser::SerializeMap for QsMap<'a, Target> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
        where T: ser::Serialize
    {
        self.1 = Some(Cow::from(key.serialize(StringSerializer)?));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        if let Some(ref key) = self.1 {
            serializer.extend_key(key);
        } else {
            return Err(Error::no_key());
        }
        self.1 = None;
        value.serialize(&mut serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
        where K: ser::Serialize, V: ser::Serialize,
    {
        let mut serializer = QsSerializer { key: self.0.key.clone(), urlencoder: self.0.urlencoder };
        serializer.extend_key(&key.serialize(StringSerializer)?);
        value.serialize(&mut serializer)
    }

}

struct StringSerializer;

impl ser::Serializer for StringSerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    serialize_as_string!{
        bool => serialize_bool,
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
    }


    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        Ok(BASE64.encode(value))
    }

    /// Returns an error.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_unit_struct(self,
                             _name: &'static str)
                             -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                              -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>
        (self,
         _name: &'static str,
         _value: &T)
         -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>
        (self,
         _name: &'static str,
         _variant_index: u32,
        _variant: &'static str,
         _value: &T)
         -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_none(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_some<T: ?Sized + ser::Serialize>
        (self,
         _value: &T)
         -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_seq(self,
                     _len: Option<usize>)
                     -> Result<Self::SerializeSeq> {
        Err(ErrorKind::Unsupported.into())
    }


    fn serialize_tuple(self,
                       _len: usize)
                       -> Result<Self::SerializeTuple> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_tuple_variant
        (self,
         _name: &'static str,
         _variant_index: u32,
         _variant: &'static str,
         _len: usize)
         -> Result<Self::SerializeTupleVariant>
    {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_map(self,
                     _len: Option<usize>)
                     -> Result<Self::SerializeMap> {
        Err(ErrorKind::Unsupported.into())

    }

    fn serialize_struct(self,
                        _name: &'static str,
                        _len: usize)
                        -> Result<Self::SerializeStruct> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_struct_variant
        (self,
         _name: &'static str,
         _variant_index: u32,
         _variant: &'static str,
         _len: usize)
         -> Result<Self::SerializeStructVariant> 
    {
        Err(ErrorKind::Unsupported.into())
    }

}

