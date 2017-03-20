#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Address {
    city: String,
    postcode: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    id: u8,
    name: String,
    phone: u32,
    address: Address,
    user_ids: Vec<u8>,
}


#[test]
fn serialize_struct() {
    let params = QueryParams {
        id: 42,
        name: "Acme".to_string(),
        phone: 12345,
        address: Address {
            city: "Carrot City".to_string(),
            postcode: "12345".to_string(),
        },
        user_ids: vec![1, 2, 3, 4],
    };

    assert_eq!(qs::to_string(&params).unwrap(),
               urlencode("\
        id=42&name=Acme&phone=12345&address[city]=Carrot+City&\
        address[postcode]=12345&user_ids[0]=1&user_ids[1]=2&\
        user_ids[2]=3&user_ids[3]=4"));
}

fn urlencode(input: &str) -> String {
    str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}

#[test]
fn serialize_option() {
    #[derive(Debug,Serialize,Deserialize,PartialEq)]
    struct Query {
        vec: Option<Vec<u8>>,
    }

    let params = "";
    let query = Query {
        vec: None,
    };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);

    let params = urlencode("vec[0]=1&vec[1]=2");
    let query = Query {
        vec: Some(vec![1,2]),
    };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);
}
