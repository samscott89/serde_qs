use axum_framework as axum;

use crate::de::Config as QsConfig;

use axum::{
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Error,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct QsQuery<T>(pub T);

#[axum::async_trait]
impl<T, B> FromRequest<B> for QsQuery<T>
where
    T: serde::de::DeserializeOwned,
    B: std::marker::Send,
{
    type Rejection = QsQueryRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(config) = Extension::<QsQueryConfig>::from_request(req)
            .await
            .unwrap_or_else(|_| Extension(QsQueryConfig::default()));
        let config: QsConfig = config.into();
        let query = req.uri().query().unwrap_or_default();
        let value = config
            .deserialize_str(query)
            .map_err(QsQueryRejection::new::<T, _>)?;
        Ok(QsQuery(value))
    }
}

impl<T> std::ops::Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct QsQueryRejection {
    error: axum::Error,
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
    pub fn new<T, E>(error: E) -> Self
    where
        E: Into<BoxError>,
    {
        QsQueryRejection {
            error: Error::new(error),
        }
    }
}

impl IntoResponse for QsQueryRejection {
    fn into_response(self) -> Response {
        let mut res = self.to_string().into_response();
        *res.status_mut() = StatusCode::BAD_REQUEST;
        res
    }
}

#[derive(Clone)]
pub struct QsQueryConfig {
    max_depth: usize,
    strict: bool,
}

impl QsQueryConfig {
    pub fn new(max_depth: usize, strict: bool) -> Self {
        Self { max_depth, strict }
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
        }
    }
}
