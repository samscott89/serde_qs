#![cfg(feature = "axum")]

extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate axum_framework as axum;
extern crate serde_qs as qs;

use axum::{extract::FromRequestParts, http::StatusCode, response::IntoResponse};
use qs::axum::{QsQuery, QsQueryConfig, QsQueryRejection};
use serde::de::Error;

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
            .extension(QsQueryConfig::new(5, false))
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
