//! A few common utility functions for encoding and decoding query strings

/// Generic serialize/deserialize functions for different delimiters
///
/// ## Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_qs::helpers::generic_delimiter::{deserialize, serialize};
///
/// #[derive(Debug, PartialEq, Deserialize, Serialize)]
/// struct Query {
///     #[serde(deserialize_with = "deserialize::<_, _, '.'>")]
///     #[serde(serialize_with = "serialize::<_, _, '.'>")]
///     values: Vec<u8>,
/// }
///
/// # fn main(){
/// let query = Query { values: vec![1, 2, 3] };
/// let serialized = serde_qs::to_string(&query).unwrap();
/// assert_eq!(
///     serialized,
///    "values=1.2.3"
/// );
/// assert_eq!(
///     serde_qs::from_str::<Query>(&serialized).unwrap(),
///     query
/// );
/// # }
/// ```
pub mod generic_delimiter {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::{borrow::Cow, str::FromStr};

    pub fn serialize<S, T, const DELIM: char>(vec: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: ToString,
    {
        let s = vec
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(&DELIM.to_string());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D, T, const DELIM: char>(deserialize: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let s: Cow<'_, str> = Deserialize::deserialize(deserialize)?;
        if s.is_empty() {
            Ok(vec![])
        } else {
            s.split(DELIM)
                .map(|x| x.parse::<T>().map_err(serde::de::Error::custom))
                .collect()
        }
    }
}

/// Serialize/deserialize comma-separated values
///
/// Equivalent to `style=form` query parameters in [OpenAPI 3.0](https://swagger.io/docs/specification/v3_0/serialization/#query-parameters)
///
/// ## Example
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Deserialize, Serialize)]
/// struct Query {
///     #[serde(with = "serde_qs::helpers::comma_separated")]
///     values: Vec<u8>,
/// }
///
/// # fn main(){
/// let query = Query { values: vec![1, 2, 3] };
/// let serialized = serde_qs::to_string(&query).unwrap();
/// assert_eq!(
///     serialized,
///    "values=1,2,3"
/// );
/// assert_eq!(
///     serde_qs::from_str::<Query>(&serialized).unwrap(),
///     query
/// );
/// # }
/// ```
pub mod comma_separated {
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S, T>(vec: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: ToString,
    {
        super::generic_delimiter::serialize::<S, T, ','>(vec, serializer)
    }

    pub fn deserialize<'de, D, T>(deserialize: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        super::generic_delimiter::deserialize::<D, T, ','>(deserialize)
    }
}

/// Serialize/deserialize pipe-delimited values
///
/// Equivalent to `style=pipeDelimited` query parameters in [OpenAPI 3.0](https://swagger.io/docs/specification/v3_0/serialization/#query-parameters)
///
/// ## Example
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Deserialize, Serialize)]
/// struct Query {
///     #[serde(with = "serde_qs::helpers::pipe_delimited")]
///     values: Vec<u8>,
/// }
///
/// # fn main(){
/// let query = Query { values: vec![1, 2, 3] };
/// let serialized = serde_qs::to_string(&query).unwrap();
/// assert_eq!(
///     serialized,
///    "values=1|2|3"
/// );
/// assert_eq!(
///     serde_qs::from_str::<Query>(&serialized).unwrap(),
///     query
/// );
/// # }
/// ```
pub mod pipe_delimited {
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S, T>(vec: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: ToString,
    {
        super::generic_delimiter::serialize::<S, T, '|'>(vec, serializer)
    }

    pub fn deserialize<'de, D, T>(deserialize: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        super::generic_delimiter::deserialize::<D, T, '|'>(deserialize)
    }
}

/// Serialize/deserialize space-delimited values
///
/// Equivalent to `style=spaceDelimited` query parameters in [OpenAPI 3.0](https://swagger.io/docs/specification/v3_0/serialization/#query-parameters).
///
/// Note that spaces are serialized as `+` in the query string since URLs do not permit
/// spaces, but deserialization will also accept the percent-encoding `%20`.
///
/// ## Example
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Deserialize, Serialize)]
/// struct Query {
///     #[serde(with = "serde_qs::helpers::space_delimited")]
///     values: Vec<u8>,
/// }
///
/// # fn main(){
/// let query = Query { values: vec![1, 2, 3] };
/// let serialized = serde_qs::to_string(&query).unwrap();
/// assert_eq!(
///     serialized,
///    "values=1+2+3"
/// );
/// assert_eq!(
///     serde_qs::from_str::<Query>(&serialized).unwrap(),
///     query
/// );
/// assert_eq!(
///     serde_qs::from_str::<Query>("values=1%202%203").unwrap(),
///     query
/// );
/// # }
/// ```
pub mod space_delimited {
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S, T>(vec: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: ToString,
    {
        super::generic_delimiter::serialize::<S, T, ' '>(vec, serializer)
    }

    pub fn deserialize<'de, D, T>(deserialize: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        super::generic_delimiter::deserialize::<D, T, ' '>(deserialize)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_empty() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Query {
            #[serde(with = "comma_separated")]
            values: Vec<u8>,
        }

        let query = Query { values: vec![] };
        let serialized = crate::to_string(&query).unwrap();
        assert_eq!(serialized, "values=");
        assert_eq!(crate::from_str::<Query>(&serialized).unwrap(), query);
        let query = Query {
            values: vec![1, 2, 3],
        };
        let serialized = crate::to_string(&query).unwrap();
        assert_eq!(serialized, "values=1,2,3");
        assert_eq!(crate::from_str::<Query>(&serialized).unwrap(), query);
    }
}
