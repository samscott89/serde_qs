//! Functionality for using `serde_qs` with `axum`.
//!
//! Enable with the `axum` feature.

use axum_framework as axum;

use std::sync::Arc;

use crate::error::Error as QsError;

use axum::{
    body::Body,
    extract::{Extension, FromRequest, FromRequestParts, RawForm, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Error,
};

/// Extract typed information from the request's query.
///
/// ## Example
///
/// ```rust
/// # use axum_framework as axum;
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
///     let app = Router::<()>::new()
///         .route("/users", get(filter_users));
/// }
pub use crate::web::QsQuery;

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
        let qs_config = Extension::<QsQueryConfig>::from_request_parts(parts, state)
            .await
            .map_or_else(|_| DEFAULT_QUERY_CONFIG.clone(), |ext| ext.0);
        let error_handler = qs_config.error_handler.clone();
        let query = parts.uri.query().unwrap_or_default();
        match qs_config.config.deserialize_str::<T>(query) {
            Ok(value) => Ok(QsQuery(value)),
            Err(err) => match error_handler {
                Some(handler) => Err((handler)(err)),
                None => Err(QsQueryRejection::new(err, StatusCode::BAD_REQUEST)),
            },
        }
    }
}

/// Extract typed information from the request's formdata.
///
/// For a `GET` request, this will extract the query string as form data.
/// Otherwise, it will extract the body of the request as form data.
///
/// By default this will use the `use_form_encoding` option from `crate::Config`.
/// If you want to use a different configuration, you can set it using
/// `QsQueryConfig` in your router.
///
/// ## Example
///
/// ```rust
/// # use axum_framework as axum;
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
///     let app = Router::<()>::new()
///         .route("/users", get(filter_users));
/// }
pub use crate::web::QsForm;

impl<T, S> FromRequest<S> for QsForm<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QsQueryRejection;

    async fn from_request(request: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();
        let qs_config = Extension::<QsQueryConfig>::from_request_parts(&mut parts, state)
            .await
            .map_or_else(|_| DEFAULT_FORM_CONFIG.clone(), |ext| ext.0);
        let error_handler = qs_config.error_handler.clone();
        // extract the form data from the request
        let request = Request::from_parts(parts, body);
        let RawForm(form_data) = RawForm::from_request(request, state)
            .await
            .map_err(|err| QsQueryRejection::new(err, StatusCode::BAD_REQUEST))?;
        match qs_config.config.deserialize_bytes::<T>(&form_data) {
            Ok(value) => Ok(QsForm(value)),
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
/// # use axum_framework as axum;
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
///     let app = Router::<()>::new()
///         .route("/users", get(filter_users))
///         .layer(Extension(QsQueryConfig::new().config(Config::default())
///             .error_handler(|err| {
///                 QsQueryRejection::new(err, StatusCode::UNPROCESSABLE_ENTITY)
///         })));
/// }
pub struct QsQueryConfig {
    config: crate::Config,
    error_handler: Option<Arc<dyn Fn(QsError) -> QsQueryRejection + Send + Sync>>,
}

static DEFAULT_QUERY_CONFIG: QsQueryConfig = QsQueryConfig {
    error_handler: None,
    config: crate::Config::new(),
};

static DEFAULT_FORM_CONFIG: QsQueryConfig = QsQueryConfig {
    error_handler: None,
    config: crate::Config::new().use_form_encoding(true),
};

impl QsQueryConfig {
    /// Create new config wrapper
    pub const fn new() -> Self {
        Self {
            config: crate::Config::new(),
            error_handler: None,
        }
    }

    pub fn config(mut self, config: crate::Config) -> Self {
        self.config = config;
        self
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

impl Default for QsQueryConfig {
    fn default() -> Self {
        Self::new()
    }
}
