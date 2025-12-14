use std::{borrow::Cow, fmt, marker::PhantomData, str::Utf8Error};

use serde::de::{self, Unexpected};

pub struct StringParsingDeserializer<'a, E> {
    value: Cow<'a, str>,
    marker: PhantomData<E>,
}

impl<E> Clone for StringParsingDeserializer<'_, E> {
    fn clone(&self) -> Self {
        StringParsingDeserializer {
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

pub fn decode_utf8(value: Cow<'_, [u8]>) -> Result<Cow<'_, str>, Utf8Error> {
    Ok(match value {
        Cow::Borrowed(bytes) => {
            let s = std::str::from_utf8(bytes)?;
            Cow::Borrowed(s)
        }
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8(bytes).map_err(|e| e.utf8_error())?),
    })
}

impl<'a, E> StringParsingDeserializer<'a, E> {
    pub fn new(value: Cow<'a, [u8]>) -> Result<Self, Utf8Error> {
        let value = decode_utf8(value)?;
        Ok(StringParsingDeserializer {
            value,
            marker: PhantomData,
        })
    }

    pub fn new_str(value: &'a str) -> Self {
        StringParsingDeserializer {
            value: Cow::Borrowed(value),
            marker: PhantomData,
        }
    }
}

macro_rules! deserialize_primitive {
    ($ty:ident, $method:ident, $visit_method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.value.parse::<$ty>() {
                Ok(val) => visitor.$visit_method(val),
                Err(_) => {
                    // if we fail to parse the value as the requested type,
                    // we'll just pass it through as a string
                    self.deserialize_any(visitor)
                }
            }
        }
    };
}

impl<'de, 'a: 'de, E> de::Deserializer<'de> for StringParsingDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }

    deserialize_primitive!(bool, deserialize_bool, visit_bool);
    deserialize_primitive!(i8, deserialize_i8, visit_i8);
    deserialize_primitive!(i16, deserialize_i16, visit_i16);
    deserialize_primitive!(i32, deserialize_i32, visit_i32);
    deserialize_primitive!(i64, deserialize_i64, visit_i64);
    deserialize_primitive!(u8, deserialize_u8, visit_u8);
    deserialize_primitive!(u16, deserialize_u16, visit_u16);
    deserialize_primitive!(u32, deserialize_u32, visit_u32);
    deserialize_primitive!(u64, deserialize_u64, visit_u64);
    deserialize_primitive!(f32, deserialize_f32, visit_f32);
    deserialize_primitive!(f64, deserialize_f64, visit_f64);

    forward_to_deserialize_any! {
        char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de, 'a: 'de, E> de::EnumAccess<'de> for StringParsingDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(unit_only)
    }
}

impl<E> fmt::Debug for StringParsingDeserializer<'_, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("StringParsingDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

pub struct UnitOnly<E> {
    marker: PhantomData<E>,
}

pub fn unit_only<T, E>(t: T) -> (T, UnitOnly<E>) {
    (
        t,
        UnitOnly {
            marker: PhantomData,
        },
    )
}

impl<'de, E> de::VariantAccess<'de> for UnitOnly<E>
where
    E: de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}
