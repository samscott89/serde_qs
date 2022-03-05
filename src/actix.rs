//! Functionality for using `serde_qs` with `actix_web`.
//!
//! Enable with the `actix4`, `actix3` or `actix2` features.

use crate::de::Config as QsConfig;
use crate::error::Error as QsError;

#[cfg(feature = "actix2")]
use actix_web2 as actix_web;
#[cfg(feature = "actix3")]
use actix_web3 as actix_web;
#[cfg(feature = "actix4")]
use actix_web4 as actix_web;

use actix_web::dev::Payload;
#[cfg(any(feature = "actix2", feature = "actix3"))]
use actix_web::HttpResponse;
use actix_web::{Error as ActixError, FromRequest, HttpRequest, ResponseError};
use futures::future::{ready, Ready};
use serde::de;
use std::fmt;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[cfg(any(feature = "actix2", feature = "actix3"))]
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
/// Extract typed information from from the request's query.
///
/// ## Example
///
/// ```rust
/// # #[macro_use] extern crate serde_derive;
/// # #[cfg(feature = "actix4")]
/// # use actix_web4 as actix_web;
/// # #[cfg(feature = "actix3")]
/// # use actix_web3 as actix_web;
/// # #[cfg(feature = "actix2")]
/// # use actix_web2 as actix_web;
/// use actix_web::{web, App, HttpResponse};
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
pub struct QsQuery<T>(T);

impl<T> Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for QsQuery<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> QsQuery<T> {
    /// Deconstruct to a inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Debug> Debug for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display> Display for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> FromRequest for QsQuery<T>
where
    T: de::DeserializeOwned,
{
    type Error = ActixError;
    type Future = Ready<Result<Self, ActixError>>;
    #[cfg(any(feature = "actix2", feature = "actix3"))]
    type Config = QsQueryConfig;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let query_config = req.app_data::<QsQueryConfig>();

        let error_handler = query_config.map(|c| c.ehandler.clone()).unwrap_or(None);

        let default_qsconfig = QsConfig::default();
        let qsconfig = query_config
            .map(|c| &c.qs_config)
            .unwrap_or(&default_qsconfig);

        let res = qsconfig
            .deserialize_str::<T>(req.query_string())
            .map(|val| Ok(QsQuery(val)))
            .unwrap_or_else(move |e| {
                let e = if let Some(error_handler) = error_handler {
                    (error_handler)(e, req)
                } else {
                    e.into()
                };

                Err(e)
            });
        ready(res)
    }
}

/// Query extractor configuration
///
/// ```rust
/// # #[macro_use] extern crate serde_derive;
/// # #[cfg(feature = "actix4")]
/// # use actix_web4 as actix_web;
/// # #[cfg(feature = "actix3")]
/// # use actix_web3 as actix_web;
/// # #[cfg(feature = "actix2")]
/// # use actix_web2 as actix_web;
/// use actix_web::{error, web, App, FromRequest, HttpResponse};
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
pub struct QsQueryConfig {
    ehandler: Option<Arc<dyn Fn(QsError, &HttpRequest) -> ActixError + Send + Sync>>,
    qs_config: QsConfig,
}

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

impl Default for QsQueryConfig {
    fn default() -> Self {
        QsQueryConfig {
            ehandler: None,
            qs_config: QsConfig::default(),
        }
    }
}
