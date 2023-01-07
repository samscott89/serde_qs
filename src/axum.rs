//! Functionality for using `serde_qs` with `axum`.
//!
//! Enable with the `axum` feature.

use axum_framework as axum;

use std::sync::Arc;

use crate::de::Config as QsConfig;
use crate::error::Error as QsError;

use axum::{
    extract::{Extension, FromRequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Error,
};

#[derive(Clone, Copy, Default)]
/// Extract typed information from from the request's query.
///
/// ## Example
///
/// ```rust
/// # extern crate axum_framework as axum;
/// use serde_qs::axum::QsQuery;
/// use serde_qs::Config;
/// use axum::{response::IntoResponse, routing::get, Router, body::Body};
///
/// #[derive(serde::Deserialize)]
/// pub struct UsersFilter {
///    id: Vec<u64>,
/// }
///
/// async fn filter_users(
///     QsQuery(info): QsQuery<UsersFilter>
/// ) -> impl IntoResponse {
///     info.id
///         .iter()
///         .map(|i| i.to_string())
///         .collect::<Vec<String>>()
///         .join(", ")
/// }
///
/// fn main() {
///     let app = Router::<(), Body>::new()
///         .route("/users", get(filter_users));
/// }
pub struct QsQuery<T>(pub T);

impl<T> std::ops::Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: std::fmt::Display> std::fmt::Display for QsQuery<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for QsQuery<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for QsQuery<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QsQueryRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Extension(qs_config) = Extension::<QsQueryConfig>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|_| Extension(QsQueryConfig::default()));
        let error_handler = qs_config.error_handler.clone();
        let config: QsConfig = qs_config.into();
        let query = parts.uri.query().unwrap_or_default();
        match config.deserialize_str::<T>(query) {
            Ok(value) => Ok(QsQuery(value)),
            Err(err) => match error_handler {
                Some(handler) => Err((handler)(err)),
                None => Err(QsQueryRejection::new(err, StatusCode::BAD_REQUEST)),
            },
        }
    }
}

#[derive(Debug)]
/// Rejection type for extractors that deserialize query strings
pub struct QsQueryRejection {
    error: axum::Error,
    status: StatusCode,
}

impl std::fmt::Display for QsQueryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to deserialize query string. Error: {}",
            self.error,
        )
    }
}

impl QsQueryRejection {
    /// Create new rejection
    pub fn new<E>(error: E, status: StatusCode) -> Self
    where
        E: Into<BoxError>,
    {
        QsQueryRejection {
            error: Error::new(error),
            status,
        }
    }
}

impl IntoResponse for QsQueryRejection {
    fn into_response(self) -> Response {
        let mut res = self.to_string().into_response();
        *res.status_mut() = self.status;
        res
    }
}

impl std::error::Error for QsQueryRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[derive(Clone)]
/// Query extractor configuration
///
/// QsQueryConfig wraps [`Config`](crate::de::Config) and implement [`Clone`]
/// for [`FromRequest`](https://docs.rs/axum/0.5/axum/extract/trait.FromRequest.html)
///
/// ## Example
///
/// ```rust
/// # extern crate axum_framework as axum;
/// use serde_qs::axum::{QsQuery, QsQueryConfig, QsQueryRejection};
/// use serde_qs::Config;
/// use axum::{
///     response::IntoResponse,
///     routing::get,
///     Router,
///     body::Body,
///     extract::Extension,
///     http::StatusCode,
/// };
/// use std::sync::Arc;
///
/// #[derive(serde::Deserialize)]
/// pub struct UsersFilter {
///    id: Vec<u64>,
/// }
///
/// async fn filter_users(
///     QsQuery(info): QsQuery<UsersFilter>
/// ) -> impl IntoResponse {
///     info.id
///         .iter()
///         .map(|i| i.to_string())
///         .collect::<Vec<String>>()
///         .join(", ")
/// }
///
/// fn main() {
///     let app = Router::<(), Body>::new()
///         .route("/users", get(filter_users))
///         .layer(Extension(QsQueryConfig::new(5, false)
///             .error_handler(|err| {
///                 QsQueryRejection::new(err, StatusCode::UNPROCESSABLE_ENTITY)
///         })));
/// }
pub struct QsQueryConfig {
    max_depth: usize,
    strict: bool,
    error_handler: Option<Arc<dyn Fn(QsError) -> QsQueryRejection + Send + Sync>>,
}

impl QsQueryConfig {
    /// Create new config wrapper
    pub fn new(max_depth: usize, strict: bool) -> Self {
        Self {
            max_depth,
            strict,
            error_handler: None,
        }
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(QsError) -> QsQueryRejection + Send + Sync + 'static,
    {
        self.error_handler = Some(Arc::new(f));
        self
    }
}

impl From<QsQueryConfig> for QsConfig {
    fn from(config: QsQueryConfig) -> Self {
        Self::new(config.max_depth, config.strict)
    }
}

impl Default for QsQueryConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            strict: true,
            error_handler: None,
        }
    }
}
