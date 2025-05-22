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

#[test]
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
    let err = qs::from_str::<Params>(s).unwrap_err();
    assert!(
        err.to_string()
            .contains("input contains invalid characters"),
        "got: {}",
        err
    );
}
