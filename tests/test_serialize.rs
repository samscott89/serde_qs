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
               "\
        id=42&name=Acme&phone=12345&address[city]=Carrot+City&\
        address[postcode]=12345&user_ids[0]=1&user_ids[1]=2&\
        user_ids[2]=3&user_ids[3]=4");
}

#[test]
fn serialize_option() {
    #[derive(Debug,Serialize,Deserialize,PartialEq)]
    struct Query {
        vec: Option<Vec<u8>>,
    }

    let params = "";
    let query = Query { vec: None };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);

    let params = "vec[0]=1&vec[1]=2";
    let query = Query { vec: Some(vec![1, 2]) };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);
}

#[test]
fn serialize_enum() {
    #[derive(Debug,Serialize,Deserialize,PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        A,
        B(bool),
        C { x: u8, y: u8 },
        D(u8, u8),
    }

    #[derive(Debug,Serialize,Deserialize,PartialEq)]
    struct Query {
        e: TestEnum,
    }

    let params = "e=a";
    let query = Query { e: TestEnum::A };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);

    let params = "e[b]=true";
    let query = Query { e: TestEnum::B(true) };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);

    let params = "e[c][x]=2&e[c][y]=3";
    let query = Query { e: TestEnum::C { x: 2, y: 3 } };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);

    let params = "e[d][0]=128&e[d][1]=1";
    let query = Query { e: TestEnum::D(128, 1) };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);
}

#[test]
fn serialize_flatten() {
    #[derive(Deserialize,Serialize,Debug, PartialEq)]
    struct Query {
        a: u8,
        #[serde(flatten)]
        common: CommonParams,
    }

    #[derive(Deserialize,Serialize,Debug, PartialEq)]
    struct CommonParams {
        limit: u64,
        offset: u64,
    }

    let params = "a=1&limit=100&offset=50";
    let query = Query { a: 1, common: CommonParams { limit: 100, offset: 50 } };
    let rec_params = qs::to_string(&query).unwrap();
    assert_eq!(rec_params, params);
}
