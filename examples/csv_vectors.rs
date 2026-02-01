use serde::{Deserialize, Serialize};
use serde_qs as qs;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Query {
    #[serde(with = "qs::helpers::comma_separated")]
    r: Vec<u8>,
    s: u8,
}

fn main() {
    let q = "s=12&r=1,2,3";
    let q: Query = qs::from_str(q).unwrap();
    println!("{:?}", q);
}

#[test]
fn deserialize_sequence() {
    let q = "s=12&r=1,2,3";
    let q: Query = qs::from_str(q).unwrap();
    let expected = Query {
        r: vec![1, 2, 3],
        s: 12,
    };
    assert_eq!(q, expected);
}
