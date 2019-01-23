//! Functionality for using `serde_qs` with `actix_web`.
//!
//! Enable with the `actix` feature.

use actix_web::FromRequest;
use actix_web::HttpRequest;
use serde::de;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::fmt;

use error::Error as QsError;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
/// Extract typed information from from the request's query.
/// `serde_qs` equivalent to `actix_web::Query`.
///
/// ## Example
///
/// ```rust
/// # extern crate actix_web;
/// # extern crate serde_qs;
/// #[macro_use] extern crate serde_derive;
/// use actix_web::{App, http};
/// use serde_qs::actix::QsQuery;
///
///#[derive(Deserialize)]
///pub struct Request {
///    id: Vec<u64>,
///}
///
/// // use `with` extractor for query info
/// // this handler get called only if request's query contains `username` field
/// // The correct request for this handler would be `/index.html?id[]=1&id[]=2"`
/// fn index(info: QsQuery<Request>) -> String {
///     format!("Request for client with list of ids={:?}", info.id)
/// }
///
/// fn main() {
///     let app = App::new().resource(
///        "/index.html",
///        |r| r.method(http::Method::GET).with(index)); // <- use `with` extractor
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

impl<T, S> FromRequest<S> for QsQuery<T>
where
    T: de::DeserializeOwned,
{
    type Config = QsQueryConfig<S>;
    type Result = Result<Self, actix_web::Error>;

    #[inline]
    fn from_request(req: &HttpRequest<S>, cfg: &Self::Config) -> Self::Result {
        let req2 = req.clone();
        let err = Rc::clone(&cfg.ehandler);
        super::from_str::<T>(req.query_string())
            .map_err(move |e| (*err)(e, &req2))
            .map(QsQuery)
    }
}

/// QsQuery extractor configuration
///
/// ```rust
/// # extern crate actix_web;
/// # extern crate serde_qs;
/// #[macro_use] extern crate serde_derive;
/// use actix_web::{error, http, App, HttpResponse, Result};
/// use serde_qs::actix::QsQuery;
///
/// #[derive(Deserialize)]
/// struct Info {
///     username: String,
/// }
///
/// /// deserialize `Info` from request's body, max payload size is 4kb
/// fn index(info: QsQuery<Info>) -> Result<String> {
///     Ok(format!("Welcome {}!", info.username))
/// }
///
/// fn main() {
///     let app = App::new().resource("/index.html", |r| {
///         r.method(http::Method::GET).with_config(index, |cfg| {
///             cfg.0.error_handler(|err, req| {
///                 // <- create custom error response
///                 error::InternalError::from_response(err.description().to_string(), HttpResponse::Conflict().finish()).into()
///             });
///         })
///     });
/// }
/// ```
pub struct QsQueryConfig<S> {
    ehandler: Rc<Fn(QsError, &HttpRequest<S>) -> actix_web::Error>,
}
impl<S> QsQueryConfig<S> {
    /// Set custom error handler
    pub fn error_handler<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(QsError, &HttpRequest<S>) -> actix_web::Error + 'static,
    {
        self.ehandler = Rc::new(f);
        self
    }
}

impl<S> Default for QsQueryConfig<S> {
    fn default() -> Self {
        QsQueryConfig {
            ehandler: Rc::new(|_, _| actix_web::error::UrlencodedError::Parse.into()),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
