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
// use url::form_urlencoded::Parse as UrlEncodedParse;
use url::form_urlencoded::parse;

/// Deserializes a `application/x-wwww-url-encoded` value from a `&[u8]`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
/// ];
/// 
/// let mut res = serde_urlencoded::from_bytes::<Vec<(String, String)>>(
///         b"bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter").unwrap();
/// res.sort();
/// assert_eq!(res, meal);
/// ```
pub fn from_bytes<T: de::Deserialize>(input: &[u8]) -> Result<T, Error> {
    T::deserialize(Deserializer::new(input))
}

/// Deserializes a `application/x-wwww-url-encoded` value from a `&str`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
/// ];
///
/// let mut res = serde_urlencoded::from_str::<Vec<(String, String)>>(
///         "bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter").unwrap();
/// res.sort();
/// assert_eq!(res, meal);
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
    // value: &'a [u8],
    // map: HashMap<Cow<'a, str>, Level<'a>>,
    // parser: Option<UrlEncodedParse<'a>>,
    iter: iter::Peekable<iter::Fuse<IntoIter<Cow<'a, str>, Level<'a>>>>,
}


// use serde::de::MapVisitor;
use std::iter;
use std::collections::hash_map::{Entry, IntoIter};

#[derive(Debug)]
enum Level<'a> {
    Nested(HashMap<Cow<'a, str>, Level<'a>>),
    Sequence(Vec<Cow<'a, str>>),
    Flat(Cow<'a, str>),
    Invalid(&'static str),
}

impl<'a> Deserializer<'a> {

    // Call this with a map, with key k, and rest should the rest of the key.
    // I.e. a[b][c]=v would be called as parse(map, "a", "b][c]", v)
    fn parse(map: &mut HashMap<Cow<'a, str>, Level<'a>>, k: Cow<'a, str>, rest: Cow<'a, str>, v: Cow<'a, str>) {
        if rest.is_empty() {
            match map.entry(k) {
                Entry::Occupied(mut o) => {
                    o.insert(Level::Invalid("Multiple values for one key"));
                },
                Entry::Vacant(vm) => {
                    vm.insert(Level::Flat(v));
                }
            }
            return;
        } else {
            // rest is not empty
            // "b][c]" =? "b", "[c]"
            let (next_key, next_rest) = split(rest, ']');
            if next_key.is_empty() {
                // key is of the form a[]
                // We assume this is at the bottom layer of nesting, otherwise we have 
                // ambiguity: a[][b]=1, a[][b]=2, a[][c]=3, a[][c] = 4
                // ==> [{b:1, c:3}, {b:2, c:4}] or 
                // ==> [{b:1, c:4}, {b:2, c:3}] ? Ordering not clear.
                if next_rest != "]" {
                    map.insert(k, Level::Invalid("unindexed nested structs is unsupported"));
                    return;
                }

                match map.entry(k) {
                    Entry::Vacant(vm) => {
                        let vec: Vec<Cow<'a, str>> = Vec::new();
                        vm.insert(Level::Sequence(vec));
                    },
                    Entry::Occupied(o) => {
                        match o.into_mut() {
                            &mut Level::Sequence(ref mut inner) => { inner.push(v); },
                            x => { *x = Level::Invalid("multiple types for one key"); }
                        }
                    }
                };
                return;
            } else {
                // assert_eq!(&rest.as_ref()[0..1], "[");
                // println!("{:?}", next_rest);
                let (e, next_rest) = split(next_rest, '[');
                assert_eq!(e, "");
                match map.entry(k).or_insert(Level::Nested(HashMap::new())) {
                    &mut Level::Nested(ref mut m) => Deserializer::parse(m, next_key, next_rest, v),
                    x => { *x = Level::Invalid(""); return; }
                    
                }
                return;
            }
        }
    }

    /// Returns a new `Deserializer`.
    pub fn new(input: &'a [u8]) -> Self {
        let mut map = HashMap::<Cow<str>, Level<'a>>::new();
        let parser = parse(input).into_iter();

        for (k, v) in parser {
            let (ldepth, rdepth) = k.chars().fold((0, 0), |(acc0, acc1), x| {
                match x {
                    '[' => (acc0+1, acc1),
                    ']' => (acc0, acc1+1),
                    _ => (acc0, acc1)
                }
            });
            debug_assert!(ldepth == rdepth);

            // Split keystring into the `root` key and the `rest`.
            // a[b][c]/// => "a", "b][c]..."
            let (root, rest) = split(k, '[');

            Deserializer::parse(&mut map, root, rest, v);        }

        // println!("{:?}", map);

        Deserializer { 
            iter: map.into_iter().fuse().peekable(),
        }
    }

    fn with_map(map: HashMap<Cow<'a, str>, Level<'a>>) -> Self {
        Deserializer {
            // value: input,
            // map: map,
            // parser: None,
            iter: map.into_iter().fuse().peekable(),
        }
    }
}

fn split<'a>(input: Cow<'a, str>, split: char) -> (Cow<'a, str>, Cow<'a, str>) {
    match input {
        Cow::Borrowed(v) => {
            let mut split2 = v.splitn(2, split);
            let s1 = split2.next().unwrap();
            let s2 = split2.next().unwrap_or("");
            (Cow::Borrowed(s1), Cow::Borrowed(s2))
        },
        Cow::Owned(v) => {
            // let v = v.into_bytes();
            let mut split_idx = v.len();
            for (idx, c) in v.chars().enumerate() {
                if c == split {
                    split_idx = idx;
                    break;
                }
            }
            // b][c] split = ], idx = 1
            if split_idx < v.len() {
                let mut v = v.into_bytes();
                let v2 = v.split_off(split_idx+1);
                v.pop();
                unsafe {
                    return (Cow::Owned(String::from_utf8_unchecked(v)),
                            Cow::Owned(String::from_utf8_unchecked(v2)))
                }
            } else {
                return (Cow::Owned(v), Cow::Owned("".to_string()))
            }
            // (Cow::Owned(v),Cow::Borrowed(""))
        }
    }
}

impl<'a, 'b> de::Deserializer for Deserializer<'a> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self)
    }

    // _serde::Deserializer::deserialize_struct(deserializer,"A", FIELDS, __Visitor)
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_map(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        // MapDeserializer
        // visitor.visit_seq(self)
        // mem::replace(self.iter))
        // let ref iter = self.iter;
        // let iter = self.iter;
        // let *self = &Deserializer::new(&[]);
        visitor.visit_seq(MapDeserializer::new(self.iter))

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
        // seq
        seq_fixed_size
        newtype_struct
        tuple_struct
        // struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

use serde::de::value::{SeqDeserializer, ValueDeserializer};


impl<'a> de::MapVisitor for Deserializer<'a> {
    type Error = Error;


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
            seed.deserialize(value.into_deserializer())
        } else {
            panic!("Somehow the list was empty after a non-empty key was returned");
        }
    }
}

struct LevelDeserializer<'a>(Level<'a>);

impl<'a> de::Deserializer for LevelDeserializer<'a> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        if let Level::Flat(x) = self.0 {
            x.into_deserializer().deserialize(visitor)
        } else {
            panic!("Could not deserialize");
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        if let Level::Nested(x) = self.0 {
            Deserializer::with_map(x).deserialize_map(visitor)
        } else {
            panic!("Could not deserialize");
        }
    }

    // _serde::Deserializer::deserialize_struct(deserializer,"A", FIELDS, __Visitor)
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {

        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        // visitor.visit_seq(self)
        if let Level::Sequence(x) = self.0 {
            SeqDeserializer::new(x.into_iter()).deserialize(visitor)
        } else {
            panic!("Could not deserialize");
        }
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
        // seq
        seq_fixed_size
        newtype_struct
        tuple_struct
        // struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

impl<'a> ValueDeserializer for Level<'a> 
{
    type Deserializer = LevelDeserializer<'a>;
    fn into_deserializer(self) -> Self::Deserializer {
        LevelDeserializer(self)
    }
}

// impl<'a> de::SeqVisitor for Deserializer<'a>
// {
//     type Error = Error;

//     fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
//         where T: de::DeserializeSeed,
//     {
//         if let Some((key, value)) = self.iter.next() {
//             let k = seed.deserialize(key.into_deserializer());
//             let v = match value {
//                 Level::Nested(x) => {
//                     seed.deserialize(&mut Deserializer::with_map(x))

//                     // seed.deserialize(&mut Deserializer::new(parse(x.as_bytes())))
//                 },
//                 Level::Sequence(x) => {
//                     seed.deserialize(SeqDeserializer::new(x.into_iter()))
//                 }
//                 Level::Flat(x) => {
//                     seed.deserialize(x.into_deserializer())
//                 },
//                 Level::Invalid(e) => {
//                     panic!("Invalid value to deserialize: {}", e);
//                 }
//             };
//             Some((k, v))
//         } else {
//             Ok(None)
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }


// fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
//      where T: de::DeserializeSeed,
//  {
//      if let Some(k) = self.0.take() {
//          seed.deserialize(k.into_deserializer()).map(Some)
//      } else if let Some(v) = self.1.take() {
//          seed.deserialize(v.into_deserializer()).map(Some)
//      } else {
//          Ok(None)
//      }
//  }

//  fn size_hint(&self) -> (usize, Option<usize>) {
//      let len = if self.0.is_some() {
//          2
//      } else if self.1.is_some() {
//          1
//      } else {
//          0
//      };
//      (len, Some(len))
//  }