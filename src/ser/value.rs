use ser::Error;
use ser::part::{PartSerializer, Sink};
use serde::ser::{Serialize, SerializeSeq, SerializeStruct};
use std::str;
use url::form_urlencoded::Serializer as UrlEncodedSerializer;
use url::form_urlencoded::Target as UrlEncodedTarget;

pub struct ValueSink<'key, 'target, Target>
    where Target: 'target + UrlEncodedTarget,
{
    urlencoder: &'target mut UrlEncodedSerializer<Target>,
    key: &'key str,
    idx: usize,
}

impl<'key, 'target, Target> ValueSink<'key, 'target, Target>
    where Target: 'target + UrlEncodedTarget,
{
    pub fn new(urlencoder: &'target mut UrlEncodedSerializer<Target>,
               key: &'key str)
               -> Self {
        ValueSink {
            urlencoder: urlencoder,
            key: key,
            idx: 0,
        }
    }
}

impl<'key, 'target, Target> Sink<(), Error> for ValueSink<'key, 'target, Target>
    where Target: 'target + UrlEncodedTarget,
{
    // type Ok = ();

    fn serialize_str(self, value: &str) -> Result<(), Error> {
        self.urlencoder.append_pair(self.key, value);
        Ok(())
    }

    fn serialize_static_str(self, value: &'static str) -> Result<(), Error> {
        self.serialize_str(value)
    }

    fn serialize_string(self, value: String) -> Result<(), Error> {
        self.serialize_str(&value)
    }

    fn serialize_none(self) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self,
                                             value: &T)
                                             -> Result<(), Error> {
        value.serialize(PartSerializer::new(self))
    }

    fn unsupported(&self) -> Error {
        Error::Custom("unsupported value".into())
    }
}


impl<'key, 'target, Target> SerializeStruct for ValueSink<'key, 'target, Target>
    where Target: 'target + UrlEncodedTarget,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              key: &'static str,
                                              value: &T)
                                              -> Result<(), Error> {
        let newk = format!("{}[{}]", self.key, key);
        let value_sink = ValueSink::new(self.urlencoder, &newk);
        value.serialize(super::part::PartSerializer::new(value_sink))
    }

    fn end(self) -> Result<Self::Ok, Error> {
        Ok(())
    }
}

impl<'key, 'target, Target> SerializeSeq for ValueSink<'key, 'target, Target>
    where Target: 'target + UrlEncodedTarget,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self,
                                                value: &T)
                                                -> Result<(), Error> {
        let newk = format!("{}[{}]", self.key, self.idx);
        self.idx += 1;
        let value_sink = ValueSink::new(self.urlencoder, &newk);
        value.serialize(super::part::PartSerializer::new(value_sink))
    }

    fn end(self) -> Result<Self::Ok, Error> {
        Ok(())
    }
}
