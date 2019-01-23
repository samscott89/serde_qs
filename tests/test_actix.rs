#![cfg(feature = "actix")]

extern crate actix_web;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

use actix_web::test::TestServer;
use qs::actix::QsQuery;
use serde::de::Error;

fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
    where D: serde::Deserializer<'de>,
          S: std::str::FromStr
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(&s).map_err(|_| D::Error::custom("could not parse string"))
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
    #[serde(deserialize_with="from_str")]
    limit: u64,
    #[serde(deserialize_with="from_str")]
    offset: u64,
    #[serde(deserialize_with="from_str")]
    remaining: bool,
}

fn my_handler(query: QsQuery<Query>) -> String {
	println!("Query: {:?}", query);
	format!("Received bars: {:?}", query.bars)
}

#[test]
fn test_qsquery() {
	let mut srv = TestServer::new(|app| {
		app.resource("/test", |h| h.with(my_handler));
	});
	let query = "/test?foo=1&bars[]=0&bars[]=1&limit=100&offset=50&remaining=true";
	let url = srv.url(query);
	let req = actix_web::client::get(url).finish().unwrap();
	let response = srv.execute(req.send()).unwrap();
	assert!(response.status().is_success());
}