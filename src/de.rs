//! Deserialization support for the `application/x-www-form-urlencoded` format.

use serde::de;

use std::collections::{
    HashMap,
};
use std::borrow::Cow;

#[doc(inline)]
pub use serde::de::value::Error;
use serde::de::value::MapDeserializer;
use std::io::Read;
use url::form_urlencoded::Parse as UrlEncodedParse;
use url::form_urlencoded::parse;

/// Deserializes a `application/x-wwww-url-encoded` value from a `&[u8]`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     serde_urlencoded::from_bytes::<Vec<(String, String)>>(
///         b"bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_bytes<T: de::Deserialize>(input: &[u8]) -> Result<T, Error> {
    T::deserialize(&mut Deserializer::new(parse(input)))
}

/// Deserializes a `application/x-wwww-url-encoded` value from a `&str`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     serde_urlencoded::from_str::<Vec<(String, String)>>(
///         "bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_str<T: de::Deserialize>(input: &str) -> Result<T, Error> {
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T, Error>
    where T: de::Deserialize, R: Read
{
    let mut buf = vec![];
    reader.read_to_end(&mut buf)
        .map_err(|e| {
            de::Error::custom(format_args!("could not read input: {}", e))
        })?;
    from_bytes(&buf)
}

/// A deserializer for the `application/x-www-form-urlencoded` format.
///
/// * Supported top-level outputs are structs, maps and sequences of pairs,
///   with or without a given length.
///
/// * Main `deserialize` methods defers to `deserialize_map`.
///
/// * Everything else but `deserialize_seq` and `deserialize_seq_fixed_size`
///   defers to `deserialize`.
pub struct Deserializer<'a> {
    inner: MapDeserializer<UrlEncodedParse<'a>, Error>,
}

impl<'a> Deserializer<'a> {
    /// Returns a new `Deserializer`.
    pub fn new(parser: UrlEncodedParse<'a>) -> Self {
        Deserializer { inner: MapDeserializer::new(parser) }
    }
}

impl<'a, 'b> de::Deserializer for &'b mut Deserializer<'a> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_map(&mut self.inner)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(&mut self.inner)
    }

    fn deserialize_seq_fixed_size<V>(self,
                                     _len: usize,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(&mut self.inner)
    }

    // _serde::Deserializer::deserialize_struct(deserializer,"A", FIELDS, __Visitor)
    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_map(FlatMapVisitor::new(self))
    }

    forward_to_deserialize! {
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
        newtype_struct
        tuple_struct
        // struct
        struct_field
        tuple
        enum
        ignored_any
    }
}


use std::marker::PhantomData;
use serde::de::MapVisitor;
use std::iter;
use std::collections::hash_map::{Iter,IntoIter};

#[derive(Debug)]
enum Level {
    Flat(String),
    Nested(String),
}
struct FlatMapVisitor<'a, 'b>
    where 'a: 'b
{
    de: &'b mut Deserializer<'a>,
    iter: iter::Peekable<iter::Fuse<IntoIter<String, Level>>>,
    // iter: iter::Peekable<iter::Fuse<Iter<'c, String, String>>>,

}

use serde::de::value::CowStrDeserializer;

impl<'a, 'b, 'c> FlatMapVisitor<'a, 'b>
    where 'a :'b
{
    fn new(de: &'b mut Deserializer<'a>) -> Self {

        let mut map = HashMap::<String, Level>::new();

        while let Ok(Some((k,v))) = de.inner.visit::<Cow<String>, Cow<String>>() {
            let (ldepth, rdepth) = k.chars().fold((0, 0), |(acc0, acc1), x| {
                match x {
                    '[' => (acc0+1, acc1),
                    ']' => (acc0, acc1+1),
                    _ => (acc0, acc1)
                }
            });
            debug_assert!(ldepth == rdepth);

            // a[b][c][d] = 1 => a, b], c][d]
            if ldepth > 1 {
                let ksplit: Vec<&str> = k.splitn(3, '[').collect();
                let a = ksplit[0];
                let b = ksplit[1];
                let c = ksplit[2];
                let x = match map.get(a.into()) {
                    Some(&Level::Flat(_)) => {
                        panic!("Tried adding a nested element to a flat level");
                    },
                    Some(&Level::Nested(ref x)) => {
                        // map.get(a) = x&b[c][d]=v 
                        format!("{}&{}[{}={}", &x, &b[..b.len()-1], &c, &v).into()
                    },
                    None => {
                        // map.insert(a, b[c][d]=v)
                        format!("{}[{}={}", &b[..b.len()-1],c, v).into()
                    }
                };
                map.insert(a.into(), Level::Nested(x));
            } else if ldepth == 1 {
                let ksplit: Vec<&str> = k.splitn(2, '[').collect();
                let a = ksplit[0];
                let b = ksplit[1];
                // k is of the form a[b]
                let x = match map.get(a.into()) {
                    Some(&Level::Flat(_)) => {
                        panic!("Tried adding a nested element to a flat level");
                    },
                    Some(&Level::Nested(ref x)) => {
                        // map.get(a) = x&b=v 
                        format!("{}&{}={}", &x, &b[..b.len()-1], &v).into()
                    },
                    None => {
                        // map.insert(a, b=v)
                        format!("{}={}", &b[..b.len()-1], &v).into()
                    }
                };
                map.insert(a.into(), Level::Nested(x));
            } else {
                // k is of the form a
                let x = match map.get(k.as_ref()) {
                    Some(_) => {   
                        panic!("Attempted to set the value of {} twice", k);
                        // map.get(a) = x&b=v 
                        // format!("{}&{}={}", &x, &b[..b.len()-1], &v).into()
                    },
                    None => {
                        v
                        // map.insert(a, b=v)
                        // format!("{}={}", &b[..b.len()-1], &v).into()
                    }
                };
                map.insert(k.into_owned(), Level::Flat(x.into_owned()));
            }
        }
        println!("Map constructed: {:?}", map);
        FlatMapVisitor {
            de: de,
            iter: map.into_iter().fuse().peekable(),
        }
    }


}

use serde::de::value::ValueDeserializer;


impl<'a, 'b, 'c> de::MapVisitor for FlatMapVisitor<'a, 'b> {
    type Error = Error;

    // __Visitor::visit_map
    // visit_map -> visit_key::<__Field> -> 
    // MapVisitor::visit_key::<__Field>()
    // -> visit_key_seed(PhantomData<__Field>)
    // -> __Field::deserialize()
    // seed.deserialize(key.into_deserializer())
    // becoes flat(seed).deserialize()

    // Swap for visit_key_seed(FlatDeserializer<__Field>)
    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: de::DeserializeSeed,
    {

        if let Some(&(ref key, _)) = self.iter.peek() {
            return seed.deserialize(key.clone().into_deserializer()).map(Some)
        };
        Ok(None)
    
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: de::DeserializeSeed,
    {
        if let Some((_, value)) = self.iter.next() {
            // seed.deserialize(value.into_deserializer())
            match value {
                Level::Flat(ref x) => {
                    seed.deserialize(x.clone().into_deserializer())
                },
                Level::Nested(ref x) => {

                    seed.deserialize(&mut Deserializer::new(parse(x.as_bytes())))
                }
            }
        } else {
            panic!("Somehow the list was empty after a non-empty key was returned");
        }
    }
}
