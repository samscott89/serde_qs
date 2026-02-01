#![cfg(feature = "axum")]

use axum::{
    extract::{FromRequest, FromRequestParts},
    http::StatusCode,
    response::IntoResponse,
};
use axum_framework as axum;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_qs::Config as QsConfig;
use serde_qs::axum::{QsForm, QsQuery, QsQueryConfig, QsQueryRejection};

fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(s).map_err(|_| D::Error::custom("could not parse string"))
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Query {
    foo: u64,
    bars: Vec<u64>,
    #[serde(flatten)]
    common: CommonParams,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct CommonParams {
    #[serde(deserialize_with = "from_str")]
    limit: u64,
    #[serde(deserialize_with = "from_str")]
    offset: u64,
    #[serde(deserialize_with = "from_str")]
    remaining: bool,
}

#[test]
fn test_default_error_handler() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test")
            .body(())
            .unwrap();
        let (mut req_parts, _) = req.into_parts();

        let e = QsQuery::<Query>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap_err();

        assert_eq!(e.into_response().status(), StatusCode::BAD_REQUEST);
    })
}

#[test]
fn test_custom_error_handler() {
    futures::executor::block_on(async {
        let req =
            axum::http::Request::builder()
                .uri("/test?foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true")
                .extension(QsQueryConfig::default().error_handler(|err| {
                    QsQueryRejection::new(err, StatusCode::UNPROCESSABLE_ENTITY)
                }))
                .body(())
                .unwrap();
        let (mut req_parts, _) = req.into_parts();
        let query = QsQuery::<Query>::from_request_parts(&mut req_parts, &()).await;

        assert!(query.is_err());
        assert_eq!(
            query.unwrap_err().into_response().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    })
}

#[test]
fn test_composite_querystring_extractor() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test?foo=1&bars[]=0&bars[]=1&limit=100&offset=50&remaining=true")
            .body(())
            .unwrap();
        let (mut req_parts, _) = req.into_parts();
        let s = QsQuery::<Query>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap();
        assert_eq!(s.foo, 1);
        assert_eq!(s.bars, vec![0, 1]);
        assert_eq!(s.common.limit, 100);
        assert_eq!(s.common.offset, 50);
        assert!(s.common.remaining);
    })
}

#[test]
fn test_default_qs_config() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test?foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true")
            .body(())
            .unwrap();
        let (mut req_parts, _) = req.into_parts();
        let e = QsQuery::<Query>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap_err();

        assert_eq!(e.into_response().status(), StatusCode::BAD_REQUEST);
    })
}

#[test]
fn test_custom_qs_config() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test?foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true")
            .extension(QsQueryConfig::new().config(serde_qs::Config::new().use_form_encoding(true)))
            .body(())
            .unwrap();

        let (mut req_parts, _) = req.into_parts();
        let s = QsQuery::<Query>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap();
        assert_eq!(s.foo, 1);
        assert_eq!(s.bars, vec![3]);
        assert_eq!(s.common.limit, 100);
        assert_eq!(s.common.offset, 50);
        assert!(s.common.remaining);
    })
}

#[test]
fn test_optional_query_none() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test")
            .body(())
            .unwrap();
        let (mut req_parts, _) = req.into_parts();

        let QsQuery(s) = QsQuery::<Option<Query>>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap();

        assert!(s.is_none());
    })
}

#[test]
fn test_optional_query_some() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test?foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true")
            .extension(QsQueryConfig::new().config(serde_qs::Config::new().use_form_encoding(true)))
            .body(())
            .unwrap();

        let (mut req_parts, _) = req.into_parts();
        let QsQuery(s) = QsQuery::<Option<Query>>::from_request_parts(&mut req_parts, &())
            .await
            .unwrap();

        let query = s.unwrap();
        assert_eq!(query.foo, 1);
        assert_eq!(query.bars, vec![3]);
        assert_eq!(query.common.limit, 100);
        assert_eq!(query.common.offset, 50);
        assert!(query.common.remaining);
    })
}

#[test]
fn test_qs_form() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test?foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true")
            .body(Default::default())
            .unwrap();

        let s = QsForm::<Query>::from_request(req, &()).await.unwrap();
        assert_eq!(s.foo, 1);
        assert_eq!(s.bars, vec![3]);
        assert_eq!(s.common.limit, 100);
        assert_eq!(s.common.offset, 50);
        assert!(s.common.remaining);
    })
}

#[test]
fn test_qs_form_post() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .uri("/test")
            .method("POST")
            .header("content-type", "application/x-www-form-urlencoded")
            .body("foo=1&bars%5B%5D=3&limit=100&offset=50&remaining=true".into())
            .unwrap();

        let s = QsForm::<Query>::from_request(req, &()).await.unwrap();
        assert_eq!(s.foo, 1);
        assert_eq!(s.bars, vec![3]);
        assert_eq!(s.common.limit, 100);
        assert_eq!(s.common.offset, 50);
        assert!(s.common.remaining);
    })
}

#[test]
fn test_qs_form_post_querystring_encoded() {
    futures::executor::block_on(async {
        let req = axum::http::Request::builder()
            .extension(
                QsQueryConfig::new().config(serde_qs::Config::new().use_form_encoding(false)),
            )
            .uri("/test")
            .method("POST")
            .header("content-type", "application/x-www-form-urlencoded")
            .body("foo=1&bars[0]=3&limit=100&offset=50&remaining=true".into())
            .unwrap();

        let s = QsForm::<Query>::from_request(req, &()).await.unwrap();
        assert_eq!(s.foo, 1);
        assert_eq!(s.bars, vec![3]);
        assert_eq!(s.common.limit, 100);
        assert_eq!(s.common.offset, 50);
        assert!(s.common.remaining);
    })
}
