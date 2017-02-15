#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;

#[test]
fn deserialize_bytes() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_bytes(b"first=23&last=42"),
               Ok(result));
}

#[test]
fn deserialize_str() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_str("first=23&last=42"),
               Ok(result));
}

#[test]
fn deserialize_reader() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_reader(b"first=23&last=42" as &[_]),
               Ok(result));
}


#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct A { b: B, c: C }
#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct B { b1: u8, b2: String }
#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct C { c1: String, c2: u8 }

#[test]
fn deserialize_struct() {
    let params = A {
      b: B {
        b1: 10,
        b2: "Ten".to_owned()
      },
      c: C {
        c1: "Seven".to_owned(),
        c2: 7
      }
    };
    let input = urlencode("b[b1]=10&b[b2]=Ten&c[c1]=Seven&c[c2]=7");
    let result: A = serde_urlencoded::from_str(&input).unwrap();
    assert_eq!(result, params);
}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}