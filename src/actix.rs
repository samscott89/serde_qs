//! Actix-web integration for `serde_qs`.
//!
//! This module provides extractors for deserializing querystrings and form data
//! with support for nested structures in Actix-web applications.
//!
//! ## Features
//!
//! - `actix4`: Support for actix-web 4.x
//! - `actix3`: Support for actix-web 3.x
//!
//! ## Why use this over actix-web's built-in extractors?
//!
//! Actix-web's `web::Query` and `web::Form` only support flat structures.
//! These extractors support nested objects and arrays using bracket notation.

use crate::Config as QsConfig;
use crate::error::Error as QsError;

#[cfg(feature = "actix3")]
use actix_web3 as actix_web;
#[cfg(feature = "actix4")]
use actix_web4 as actix_web;

#[cfg(feature = "actix3")]
use actix_web::HttpResponse;
use actix_web::dev::Payload;
use actix_web::{Error as ActixError, FromRequest, HttpRequest, ResponseError, web};
use futures::StreamExt;
use futures::future::{FutureExt, LocalBoxFuture, Ready, ready};
use serde::de;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::Arc;

#[cfg(feature = "actix3")]
impl ResponseError for QsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().finish()
    }
}

#[cfg(feature = "actix4")]
impl ResponseError for QsError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }
}

/// Extract typed information from the request's query string.
///
/// A drop-in replacement for `actix_web::web::Query` that supports nested structures.
/// Use this when you need to deserialize querystrings with arrays or nested objects.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "actix4")]
/// # use actix_web4 as actix_web;
/// # #[cfg(feature = "actix3")]
/// # use actix_web3 as actix_web;
/// use actix_web::{web, App, HttpResponse};
/// use serde::Deserialize;
/// use serde_qs::actix::QsQuery;
///
/// #[derive(Deserialize)]
/// pub struct UsersFilter {
///    id: Vec<u64>,
/// }
///
/// // Use `QsQuery` extractor for query information.
/// // The correct request for this handler would be `/users?id[]=1124&id[]=88"`
/// async fn filter_users(info: QsQuery<UsersFilter>) -> HttpResponse {
///     HttpResponse::Ok().body(
///         info.id.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
///     )
/// }
///
/// fn main() {
///     let app = App::new().service(
///        web::resource("/users")
///            .route(web::get().to(filter_users)));
/// }
/// ```
pub use crate::web::QsQuery;

impl<T> FromRequest for QsQuery<T>
where
    T: de::DeserializeOwned,
{
    type Error = ActixError;
    type Future = Ready<Result<Self, ActixError>>;
    #[cfg(feature = "actix3")]
    type Config = QsQueryConfig;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let query_config = req
            .app_data::<QsQueryConfig>()
            .unwrap_or(&DEFAULT_QUERY_CONFIG);

        let res = query_config
            .qs_config
            .deserialize_str::<T>(req.query_string())
            .map(|val| Ok(QsQuery(val)))
            .unwrap_or_else(move |e| {
                let e = if let Some(error_handler) = &query_config.ehandler {
                    (error_handler)(e, req)
                } else {
                    e.into()
                };

                Err(e)
            });
        ready(res)
    }
}

/// Extract typed information from the request's form data.
///
/// A drop-in replacement for `actix_web::web::Form` that supports nested structures.
/// Use this when you need to deserialize form data with arrays or nested objects.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "actix4")]
/// # use actix_web4 as actix_web;
/// # #[cfg(feature = "actix3")]
/// # use actix_web3 as actix_web;
/// use actix_web::{web, App, HttpResponse};
/// use serde::Deserialize;
/// use serde_qs::actix::QsForm;
///
/// #[derive(Debug, Deserialize)]
/// pub struct UsersFilter {
///    id: Vec<u64>,
/// }
///
/// // Use `QsForm` extractor for Form information.
/// // Content-Type: application/x-www-form-urlencoded
/// // The correct request payload for this handler would be `id[]=1124&id[]=88`
/// async fn filter_users(info: QsForm<UsersFilter>) -> HttpResponse {
///     HttpResponse::Ok().body(
///         info.id.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
///     )
/// }
///
/// fn main() {
///     let app = App::new().service(
///        web::resource("/users")
///            .route(web::get().to(filter_users)));
/// }
/// ```
pub use crate::web::QsForm;

impl<T> FromRequest for QsForm<T>
where
    T: DeserializeOwned + Debug,
{
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self, ActixError>>;
    #[cfg(feature = "actix3")]
    type Config = QsQueryConfig;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let mut stream = payload.take();
        let req_clone = req.clone();

        let query_config: QsQueryConfig = req
            .app_data::<QsQueryConfig>()
            .unwrap_or(&DEFAULT_FORM_CONFIG)
            .clone();
        async move {
            let mut bytes = web::BytesMut::new();

            while let Some(item) = stream.next().await {
                bytes.extend_from_slice(&item.unwrap());
            }

            query_config
                .qs_config
                .deserialize_bytes::<T>(&bytes)
                .map(|val| Ok(QsForm(val)))
                .unwrap_or_else(|e| {
                    let e = if let Some(error_handler) = &query_config.ehandler {
                        (error_handler)(e, &req_clone)
                    } else {
                        e.into()
                    };

                    Err(e)
                })
        }
        .boxed_local()
    }
}

type ActixErrorHandler = Option<Arc<dyn Fn(QsError, &HttpRequest) -> ActixError + Send + Sync>>;

/// Configuration for query and form extractors.
///
/// Allows customization of error handling and serialization parameters
/// for both `QsQuery` and `QsForm` extractors.
///
/// ```rust
/// # #[cfg(feature = "actix4")]
/// # use actix_web4 as actix_web;
/// # #[cfg(feature = "actix3")]
/// # use actix_web3 as actix_web;
/// use actix_web::{error, web, App, FromRequest, HttpResponse};
/// use serde::Deserialize;
/// use serde_qs::actix::QsQuery;
/// use serde_qs::Config as QsConfig;
/// use serde_qs::actix::QsQueryConfig;
///
/// #[derive(Deserialize)]
/// struct Info {
///     username: String,
/// }
///
/// /// deserialize `Info` from request's querystring
/// async fn index(info: QsQuery<Info>) -> HttpResponse {
///     HttpResponse::Ok().body(
///         format!("Welcome {}!", info.username)
///     )
/// }
///
/// fn main() {
/// let qs_config = QsQueryConfig::default()
///     .error_handler(|err, req| {  // <- create custom error response
///     error::InternalError::from_response(
///         err, HttpResponse::Conflict().finish()).into()
///     })
///     .qs_config(QsConfig::default());
///
/// let app = App::new().service(
///         web::resource("/index.html").app_data(qs_config)
///             .route(web::post().to(index))
///     );
/// }
/// ```
#[derive(Clone, Default)]
pub struct QsQueryConfig {
    ehandler: ActixErrorHandler,
    qs_config: QsConfig,
}

static DEFAULT_QUERY_CONFIG: QsQueryConfig = QsQueryConfig {
    ehandler: None,
    qs_config: QsConfig::new(),
};

static DEFAULT_FORM_CONFIG: QsQueryConfig = QsQueryConfig {
    ehandler: None,
    qs_config: QsConfig::new().use_form_encoding(true),
};

impl QsQueryConfig {
    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(QsError, &HttpRequest) -> ActixError + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set custom serialization parameters
    pub fn qs_config(mut self, config: QsConfig) -> Self {
        self.qs_config = config;
        self
    }
}
