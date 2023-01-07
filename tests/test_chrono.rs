extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

#[test]
fn test_dates() {
    use chrono::prelude::*;
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Params {
        date_time: DateTime<FixedOffset>,
    }

    #[allow(deprecated)]
    let params = Params {
        date_time: FixedOffset::east(9 * 3600)
            .ymd(2014, 11, 28)
            .and_hms_nano(21, 45, 59, 324310806),
    };

    let s = qs::to_string(&params).unwrap();
    assert_eq!(s, "date_time=2014-11-28T21%3A45%3A59.324310806%2B09%3A00");

    let data: Params = qs::from_str(&s).unwrap();
    assert_eq!(data, params);
}

/// Curious what happens if we _don't_ urlencode the string parameter
#[test]
#[should_panic]
fn test_improperly_encoded_dates() {
    use chrono::prelude::*;
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Params {
        date_time: DateTime<FixedOffset>,
    }

    #[allow(deprecated)]
    let _expected = Params {
        date_time: FixedOffset::east(9 * 3600)
            .ymd(2014, 11, 28)
            .and_hms_nano(21, 45, 59, 324310806),
    };

    let s = "date_time=2014-11-28T21:45:59.324310806+09:00";
    let _data: Params = qs::from_str(s).unwrap();
    // assert_eq!(data, params);
}
