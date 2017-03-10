use ser::{Error};
use serde::ser::{SerializeSeq, SerializeStruct};
use ser::part::Sink;
use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

pub enum Key<'key> {
    Static(&'static str),
    Dynamic(Cow<'key, str>),
}

impl<'key> Deref for Key<'key> {
    type Target = str;

    fn deref(&self) -> &str {
        match *self {
            Key::Static(key) => key,
            Key::Dynamic(ref key) => key,
        }
    }
}

impl<'key> From<Key<'key>> for Cow<'static, str> {
    fn from(key: Key<'key>) -> Self {
        match key {
            Key::Static(key) => key.into(),
            Key::Dynamic(key) => key.into_owned().into(),
        }
    }
}

pub struct KeySink<End> {
    end: End,
}

impl<End, Ok> KeySink<End>
    where End: for<'key> FnOnce(Key<'key>) -> Result<Ok, Error>
{
    pub fn new(end: End) -> Self {
        KeySink { end: end }
    }
}

impl<End, Ok> Sink<Ok, Error> for KeySink<End>
    where End: for<'key> FnOnce(Key<'key>) -> Result<Ok, Error>
{

    fn serialize_static_str(self,
                            value: &'static str)
                            -> Result<Ok, Error> {
        (self.end)(Key::Static(value))
    }

    fn serialize_str(self, value: &str) -> Result<Ok, Error> {
        (self.end)(Key::Dynamic(value.into()))
    }

    fn serialize_string(self, value: String) -> Result<Ok, Error> {
        (self.end)(Key::Dynamic(value.into()))
    }

    fn serialize_none(self) -> Result<Ok, Error> {
        Err(self.unsupported())
    }

    fn serialize_some<T: ?Sized + Serialize>(self,
                                             _value: &T)
                                             -> Result<Ok, Error> {
        Err(self.unsupported())
    }

    fn unsupported(&self) -> Error {
        Error::Custom("unsupported key".into())
    }
}

impl<End, Ok> SerializeStruct for KeySink<End>
where End: for<'key> FnOnce(Key<'key>) -> Result<Ok, Error>
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                                   _key: &'static str,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        Err(self.unsupported())
    }

    fn end(self) -> Result<Self::Ok, Error> {
        Err(self.unsupported())
    }
}

impl<End, Ok> SerializeSeq for KeySink<End>
where End: for<'key> FnOnce(Key<'key>) -> Result<Ok, Error>
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        Err(self.unsupported())
    }

    fn end(self) -> Result<Self::Ok, Error> {
        Err(self.unsupported())
    }
}



