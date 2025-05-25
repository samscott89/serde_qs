//! Serde support for querystring-style strings
//!
//! This library provides serialization and deserialization of querystrings
//! with support for arbitrarily nested structures. Unlike `serde_urlencoded`,
//! which only handles flat key-value pairs, `serde_qs` supports complex nested
//! data using bracket notation (e.g., `user[name]=John&user[age]=30`).
//!
//! ## Why use `serde_qs`?
//!
//! - **Nested structure support**: Serialize/deserialize complex structs and maps
//! - **Array support**: Handle vectors and sequences with indexed notation
//! - **Framework integration**: Built-in support for Actix-web, Axum, and Warp
//! - **Compatible syntax**: Works with `qs` (JavaScript) and Rack (Ruby)
//!
//!
//! ## Basic Usage
//!
//! ```
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_qs as qs;
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Address {
//!     city: String,
//!     postcode: String,
//! }
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct QueryParams {
//!     id: u8,
//!     name: String,
//!     address: Address,
//!     phone: u32,
//!     user_ids: Vec<u8>,
//! }
//!
//! # fn main() {
//! let params = QueryParams {
//!     id: 42,
//!     name: "Acme".to_string(),
//!     phone: 12345,
//!     address: Address {
//!         city: "Carrot City".to_string(),
//!         postcode: "12345".to_string(),
//!     },
//!     user_ids: vec![1, 2, 3, 4],
//! };
//! let rec_params: QueryParams = qs::from_str("\
//!     name=Acme&id=42&phone=12345&address[postcode]=12345&\
//!     address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&\
//!     user_ids[2]=3&user_ids[3]=4")
//!     .unwrap();
//! assert_eq!(rec_params, params);
//!
//! # }
//! ```
//!
//! ## Supported Types
//!
//! `serde_qs` supports all serde-compatible types:
//!
//! - **Primitives**: strings, integers (u8-u64, i8-i64), floats (f32, f64), booleans
//! - **Strings**: UTF-8 strings (invalid UTF-8 handling configurable)
//! - **Bytes**: `Vec<u8>` and `&[u8]` for raw binary data
//! - **Collections**: `Vec<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`, arrays
//! - **Options**: `Option<T>` (missing values deserialize to `None`)
//! - **Structs**: Named and tuple structs with nested fields
//! - **Enums**: Externally tagged, internally tagged, and untagged representations
//!
//! Note: Top-level types must be structs or maps. Primitives and sequences
//! cannot be deserialized at the top level. And untagged representations
//! have some limitations (see [Flatten Workaround](#flatten-workaround) section).
//!
//! ## Query-String vs Form Encoding
//!
//! By default, `serde_qs` uses **query-string encoding** which is more permissive:
//! - Spaces encoded as `+`
//! - Minimal percent-encoding (brackets remain unencoded)
//! - Example: `name=John+Doe&items[0]=apple`
//!
//! The main benefit of query-string encoding is that it allows for more compact
//! representations of nested structures, and supports square brackets in
//! key names.
//!
//! **Form encoding** (`application/x-www-form-urlencoded`) is stricter:
//! - Spaces encoded as `%20`
//! - Most special characters percent-encoded
//! - Example: `name=John%20Doe&items%5B0%5D=apple`
//!
//! Form encoding is useful for compability with HTML forms and other
//! applications that eagerly encode brackets.
//!
//! Configure encoding mode:
//! ```rust
//! use serde_qs::Config;
//!
//! // Use form encoding
//! # fn main() -> Result<(), serde_qs::Error> {
//! # let my_struct = ();
//! let config = Config::new().use_form_encoding(true);
//! let qs = config.serialize_string(&my_struct)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## UTF-8 Handling
//!
//! By default, `serde_qs` requires valid UTF-8 in string values. If your data
//! may contain non-UTF-8 bytes, consider serializing to `Vec<u8>` instead of
//! `String`. Non-UTF-8 bytes in ignored fields will not cause errors.
//!
//! ```rust
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Data {
//!     // This field can handle raw bytes
//!     raw_data: Vec<u8>,
//!     
//!     // This field requires valid UTF-8
//!     text: String,
//! }
//! ```
//!
//! ## Helpers for Common Scenarios
//!
//! The `helpers` module provides utilities for common patterns when working with
//! querystrings, particularly for handling delimited values within a single parameter.
//!
//! ### Comma-Separated Values
//!
//! Compatible with OpenAPI 3.0 `style=form` parameters:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Query {
//!     #[serde(with = "serde_qs::helpers::comma_separated")]
//!     ids: Vec<u64>,
//! }
//!
//! # fn main() {
//! // Deserialize from comma-separated string
//! let query: Query = serde_qs::from_str("ids=1,2,3,4").unwrap();
//! assert_eq!(query.ids, vec![1, 2, 3, 4]);
//!
//! // Serialize back to comma-separated
//! let qs = serde_qs::to_string(&query).unwrap();
//! assert_eq!(qs, "ids=1,2,3,4");
//! # }
//! ```
//!
//! ### Other Delimiters
//!
//! Also supports pipe (`|`) and space delimited values:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Query {
//!     #[serde(with = "serde_qs::helpers::pipe_delimited")]
//!     tags: Vec<String>,
//!     #[serde(with = "serde_qs::helpers::space_delimited")]
//!     words: Vec<String>,
//! }
//!
//! # fn main() {
//! let query: Query = serde_qs::from_str("tags=foo|bar|baz&words=hello+world").unwrap();
//! assert_eq!(query.tags, vec!["foo", "bar", "baz"]);
//! assert_eq!(query.words, vec!["hello", "world"]);
//! # }
//! ```
//!
//! ### Custom Delimiters
//!
//! For other delimiters, use the generic helper:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_qs::helpers::generic_delimiter::{deserialize, serialize};
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Query {
//!     #[serde(deserialize_with = "deserialize::<_, _, '.'>")]
//!     #[serde(serialize_with = "serialize::<_, _, '.'>")]
//!     versions: Vec<u8>,
//! }
//!
//! # fn main() {
//! let query: Query = serde_qs::from_str("versions=1.2.3").unwrap();
//! assert_eq!(query.versions, vec![1, 2, 3]);
//! # }
//! ```
//!
//! ## Flatten workaround
//!
//! A current [known limitation](https://github.com/serde-rs/serde/issues/1183)
//! in `serde` is deserializing `#[serde(flatten)]` structs for formats which
//! are not self-describing. This includes query strings: `12` can be an integer
//! or a string, for example.
//!
//! We suggest the following workaround:
//!
//! ```
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_qs as qs;
//! extern crate serde_with;
//!
//! use serde_with::{serde_as, DisplayFromStr};
//!
//! #[derive(Deserialize, Serialize, Debug, PartialEq)]
//! struct Query {
//!     a: u8,
//!     #[serde(flatten)]
//!     common: CommonParams,
//! }
//!
//! #[serde_as]
//! #[derive(Deserialize, Serialize, Debug, PartialEq)]
//! struct CommonParams {
//!     #[serde_as(as = "DisplayFromStr")]
//!     limit: u64,
//!     #[serde_as(as = "DisplayFromStr")]
//!     offset: u64,
//!     #[serde_as(as = "DisplayFromStr")]
//!     remaining: bool,
//! }
//!
//! fn main() {
//!     let params = "a=1&limit=100&offset=50&remaining=true";
//!     let query = Query { a: 1, common: CommonParams { limit: 100, offset: 50, remaining: true } };
//!     let rec_query: Result<Query, _> = qs::from_str(params);
//!     assert_eq!(rec_query.unwrap(), query);
//! }
//! ```
//!
//! ## Use with `actix_web` extractors
//!
//! The `actix4`, `actix3` or `actix2` features enable the use of `serde_qs::actix::QsQuery`, which
//! is a direct substitute for the `actix_web::Query` and can be used as an extractor:
//!
//! ```ignore
//! fn index(info: QsQuery<Info>) -> Result<String> {
//!     Ok(format!("Welcome {}!", info.username))
//! }
//! ```
//!
//! Support for `actix-web 4.0` is available via the `actix4` feature.
//! Support for `actix-web 3.0` is available via the `actix3` feature.
//! Support for `actix-web 2.0` is available via the `actix2` feature.
//!
//! ## Use with `warp` filters
//!
//! The `warp` feature enables the use of `serde_qs::warp::query()`, which
//! is a substitute for the `warp::query::query()` filter and can be used like this:
//!
//! ```ignore
//! serde_qs::warp::query(Config::default())
//!     .and_then(|info| async move {
//!         Ok::<_, Rejection>(format!("Welcome {}!", info.username))
//!     })
//!     .recover(serde_qs::warp::recover_fn);
//! ```
//!

#[macro_use]
extern crate serde;

#[cfg(any(feature = "actix4", feature = "actix3"))]
pub mod actix;

#[cfg(feature = "actix")]
compile_error!(
    r#"The `actix` feature was removed in v0.9 due to the proliferation of actix versions.
You must now specify the desired actix version by number.

E.g.

serde_qs = { version = "0.9", features = ["actix4"] }

"#
);

#[cfg(feature = "actix2")]
compile_error!(
    r#"The `actix2` feature was removed in v0.13 due to CI issues and minimal interest in continuing support"#
);

mod config;
#[doc(inline)]
pub use config::Config;
mod de;
mod error;
pub mod helpers;
mod ser;

#[doc(inline)]
pub use de::QsDeserializer as Deserializer;
#[doc(inline)]
pub use de::{from_bytes, from_str};

pub use error::Error;
#[doc(inline)]
pub use ser::{to_string, to_writer, QsSerializer as Serializer};

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "warp")]
pub mod warp;

#[cfg(feature = "indexmap")]
mod indexmap {
    use std::borrow::Borrow;

    pub use indexmap::map::Entry;
    pub use indexmap::IndexMap as Map;

    pub fn remove<K, V, Q>(map: &mut Map<K, V>, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + std::hash::Hash + Eq,
        Q: ?Sized + std::hash::Hash + Eq,
    {
        map.shift_remove(key)
    }

    pub fn pop_first<K, V>(map: &mut Map<K, V>) -> Option<(K, V)> {
        map.shift_remove_index(0)
    }
}

#[cfg(feature = "indexmap")]
pub(crate) use crate::indexmap as map;

#[cfg(not(feature = "indexmap"))]
mod btree_map {
    use std::borrow::Borrow;
    pub use std::collections::btree_map::Entry;
    pub use std::collections::BTreeMap as Map;

    pub fn remove<K, V, Q>(map: &mut Map<K, V>, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord,
    {
        map.remove(key)
    }

    pub fn pop_first<K: Ord, V>(map: &mut Map<K, V>) -> Option<(K, V)> {
        map.pop_first()
    }
}

#[cfg(not(feature = "indexmap"))]
pub(crate) use crate::btree_map as map;
