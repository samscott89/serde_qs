//! Serde support for querystring-style strings
//!
//! Querystrings are not formally defined and loosely take the form of
//! _nested_ urlencoded queries.
//!
//! This library aims for compatability with the syntax of
//! [qs](https://github.com/ljharb/qs) and also of the
//! [`Rack::Utils::parse_nested_query`](http://www.rubydoc.info/github/rack/rack/Rack/Utils#parse_nested_query-class_method)
//! implementation.
//!
//! For users who do *not* require nested URL parameters, it is highly
//! recommended that the `serde_urlencoded` crate is used instead, which
//! will almost certainly perform better for deserializing simple inputs.
//!
//! ## Supported Types
//!
//! At the **top level**, `serde_qs` only supports `struct`, `map`, and `enum`.
//! These are the only top-level structs which can be de/serialized since
//! Querystrings rely on having a (key, value) pair for each field, which
//! necessitates this kind of structure.
//!
//! However, after the top level you should find all supported types can be
//! de/serialized.
//!
//! Note that integer keys are reserved for array indices. That is, a string of
//! the form `a[0]=1&a[1]=3` will deserialize to the ordered sequence `a =
//! [1,3]`.
//!
//! ## Usage
//!
//! See the examples folder for a more detailed introduction.
//!
//! Serializing/Deserializing is designed to work with maps and structs.
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
//! ## Strict vs Non-Strict modes
//!
//! `serde_qs` supports two operating modes, which can be specified using
//! [`Config`](struct.Config.html).
//! Strict mode has two parts:
//! - how `serde_qs` handles square brackets
//! - how `serde_qs` handles invalid UTF-8 percent decoded characters
//!
//! ### Square Brackets
//!
//! Technically, square brackets should be encoded in URLs as `%5B` and `%5D`.
//! However, they are often used in their raw format to specify querystrings
//! such as `a[b]=123`.
//!
//! In strict mode, `serde_qs` will only tolerate unencoded square brackets
//! to denote nested keys. So `a[b]=123` will decode as `{"a": {"b": 123}}`.
//! This means that encoded square brackets can actually be part of the key.
//! `a[b%5Bc%5D]=123` becomes `{"a": {"b[c]": 123}}`.
//!
//! However, since some implementations will automatically encode everything
//! in the URL, we also have a non-strict mode. This means that `serde_qs`
//! will assume that any encoded square brackets in the string were meant to
//! be taken as nested keys. From the example before, `a[b%5Bc%5D]=123` will
//! now become `{"a": {"b": {"c": 123 }}}`.
//!
//! Non-strict mode can be useful when, as said before, some middleware
//! automatically encodes the brackets. But care must be taken to avoid
//! using keys with square brackets in them, or unexpected things can
//! happen.
//!
//! ### Invalid UTF-8 Percent Encodings
//!
//! Sometimes querystrings may have percent-encoded data which does not decode
//! to UTF-8. In some cases it is useful for this to cause errors, which is how
//! `serde_qs` works in strict mode (the default). Whereas in other cases it
//! can be useful to just replace such data with the unicode replacement
//! character (� `U+FFFD`), which is how `serde_qs` works in non-strict mode.
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

mod de;
mod error;
mod ser;
pub(crate) mod utils;

#[doc(inline)]
pub use de::{from_bytes, from_str};
#[doc(inline)]
pub use de::{Config, QsDeserializer as Deserializer};
pub use error::Error;
#[doc(inline)]
pub use ser::{to_string, to_writer, Serializer};

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "warp")]
pub mod warp;

#[cfg(feature = "indexmap")]
mod indexmap {
    use std::borrow::Borrow;

    pub use indexmap::map::Entry;
    pub use indexmap::IndexMap as Map;

    pub fn remove_entry<K, V, Q>(map: &mut Map<K, V>, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + std::hash::Hash + Eq,
        Q: ?Sized + std::hash::Hash + Eq,
    {
        map.shift_remove_entry(key)
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

    pub fn remove_entry<K, V, Q>(map: &mut Map<K, V>, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord,
    {
        map.remove_entry(key)
    }

    pub fn pop_first<K: Ord, V>(map: &mut Map<K, V>) -> Option<(K, V)> {
        map.pop_first()
    }
}

#[cfg(not(feature = "indexmap"))]
pub(crate) use crate::btree_map as map;
