use crate::{de::Config as QsConfig, error};
use serde::de;
use std::sync::Arc;
use warp::{reject::Reject, Filter, Rejection};

impl Reject for error::Error {}

/// Extract typed information from from the request's query.
///
/// ## Example
///
/// ```rust
/// # #[macro_use] extern crate serde_derive;
/// use warp::Filter;
/// use serde_qs::Config;
///
/// #[derive(Deserialize)]
/// pub struct UsersFilter {
///    id: Vec<u64>,
/// }
///
/// fn main() {
///     let filter = serde_qs::warp::query(Config::default())
///         .and_then(|info: UsersFilter| async move {
///             Ok::<_, warp::Rejection>(
///                 info.id.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
///             )
///         });
/// }
/// ```
pub fn query<T>(config: QsConfig) -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: de::DeserializeOwned,
{
    let config = Arc::new(config);

    warp::query::raw().and_then(move |query: String| {
        let config = Arc::clone(&config);

        async move {
            config
                .deserialize_str(query.as_str())
                .map_err(Rejection::from)
        }
    })
}
