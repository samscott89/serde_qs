use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Simple data structures for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleStruct {
    id: u32,
    name: String,
    active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleWithOption {
    id: u32,
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleVecWrapper {
    items: Vec<u32>,
}

// Complex/nested data structures for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Address {
    city: String,
    street: String,
    postcode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryParams {
    id: u8,
    name: String,
    phone: u32,
    address: Address,
    user_ids: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepNested {
    level1: Level1,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Level1 {
    level2: Level2,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Level2 {
    level3: Level3,
    config: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Level3 {
    value: String,
    flags: Vec<bool>,
}

fn serialize_simple_struct(c: &mut Criterion) {
    let data = SimpleStruct {
        id: 42,
        name: "test_user".to_string(),
        active: true,
    };

    c.bench_function("serialize_simple_struct", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_simple_with_option_some(c: &mut Criterion) {
    let data = SimpleWithOption {
        id: 123,
        name: "user_with_email".to_string(),
        email: Some("user@example.com".to_string()),
    };

    c.bench_function("serialize_simple_with_option_some", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_simple_with_option_none(c: &mut Criterion) {
    let data = SimpleWithOption {
        id: 456,
        name: "user_without_email".to_string(),
        email: None,
    };

    c.bench_function("serialize_simple_with_option_none", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_simple_vec(c: &mut Criterion) {
    let data = SimpleVecWrapper {
        items: vec![1u32, 2, 3, 4, 5],
    };

    c.bench_function("serialize_simple_vec", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_simple_hashmap(c: &mut Criterion) {
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());
    data.insert("key3".to_string(), "value3".to_string());

    c.bench_function("serialize_simple_hashmap", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

// Complex/nested serialization benchmarks
fn serialize_nested_struct(c: &mut Criterion) {
    let data = QueryParams {
        id: 42,
        name: "Acme".to_string(),
        phone: 12345,
        address: Address {
            city: "Carrot City".to_string(),
            street: "Special-Street* No. 11".to_string(),
            postcode: "12345".to_string(),
        },
        user_ids: vec![1, 2, 3, 4],
    };

    c.bench_function("serialize_nested_struct", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_deep_nested(c: &mut Criterion) {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("author".to_string(), "test".to_string());

    let mut config = HashMap::new();
    config.insert("max_retry".to_string(), 3);
    config.insert("timeout".to_string(), 30);

    let data = DeepNested {
        level1: Level1 {
            level2: Level2 {
                level3: Level3 {
                    value: "deep_value".to_string(),
                    flags: vec![true, false, true],
                },
                config,
            },
            tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        },
        metadata,
    };

    c.bench_function("serialize_deep_nested", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn serialize_large_vec(c: &mut Criterion) {
    let data = SimpleVecWrapper {
        items: (0..100).collect(),
    };

    c.bench_function("serialize_large_vec", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

// Deserialization benchmarks
fn deserialize_simple_struct(c: &mut Criterion) {
    let query = "id=42&name=test_user&active=true";

    c.bench_function("deserialize_simple_struct", |b| {
        b.iter(|| {
            let _: SimpleStruct = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_simple_with_option_some(c: &mut Criterion) {
    let query = "id=123&name=user_with_email&email=user%40example.com";

    c.bench_function("deserialize_simple_with_option_some", |b| {
        b.iter(|| {
            let _: SimpleWithOption = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_simple_with_option_none(c: &mut Criterion) {
    let query = "id=456&name=user_without_email";

    c.bench_function("deserialize_simple_with_option_none", |b| {
        b.iter(|| {
            let _: SimpleWithOption = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_simple_vec(c: &mut Criterion) {
    let query = "items[0]=1&items[1]=2&items[2]=3&items[3]=4&items[4]=5";

    c.bench_function("deserialize_simple_vec", |b| {
        b.iter(|| {
            let _: SimpleVecWrapper = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_simple_hashmap(c: &mut Criterion) {
    let query = "key1=value1&key2=value2&key3=value3";

    c.bench_function("deserialize_simple_hashmap", |b| {
        b.iter(|| {
            let _: HashMap<String, String> = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

// Complex/nested deserialization benchmarks
fn deserialize_nested_struct(c: &mut Criterion) {
    let query = "id=42&name=Acme&phone=12345&address[city]=Carrot+City&address[street]=Special-Street*+No.+11&address[postcode]=12345&user_ids[0]=1&user_ids[1]=2&user_ids[2]=3&user_ids[3]=4";

    c.bench_function("deserialize_nested_struct", |b| {
        b.iter(|| {
            let _: QueryParams = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_deep_nested(c: &mut Criterion) {
    let query = "level1[level2][level3][value]=deep_value&level1[level2][level3][flags][0]=true&level1[level2][level3][flags][1]=false&level1[level2][level3][flags][2]=true&level1[level2][config][max_retry]=3&level1[level2][config][timeout]=30&level1[tags][0]=tag1&level1[tags][1]=tag2&level1[tags][2]=tag3&metadata[version]=1.0&metadata[author]=test";

    c.bench_function("deserialize_deep_nested", |b| {
        b.iter(|| {
            let _: DeepNested = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn deserialize_large_vec(c: &mut Criterion) {
    // Generate query string for 100 items: items[0]=0&items[1]=1&...&items[99]=99
    let query = (0..100)
        .map(|i| format!("items[{}]={}", i, i))
        .collect::<Vec<_>>()
        .join("&");

    c.bench_function("deserialize_large_vec", |b| {
        b.iter(|| {
            let _: SimpleVecWrapper = serde_qs::from_str(black_box(&query)).unwrap();
        })
    });
}

// Comparison benchmarks: serde_qs vs serde_urlencoded
// Note: Only flat structures work with serde_urlencoded
fn comparison_simple_struct_serde_qs(c: &mut Criterion) {
    let data = SimpleStruct {
        id: 42,
        name: "test_user".to_string(),
        active: true,
    };

    c.bench_function("comparison_simple_struct_serde_qs_serialize", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn comparison_simple_struct_serde_urlencoded(c: &mut Criterion) {
    let data = SimpleStruct {
        id: 42,
        name: "test_user".to_string(),
        active: true,
    };

    c.bench_function("comparison_simple_struct_serde_urlencoded_serialize", |b| {
        b.iter(|| {
            serde_urlencoded::to_string(black_box(&data)).unwrap()
        })
    });
}

fn comparison_hashmap_serde_qs(c: &mut Criterion) {
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());
    data.insert("key3".to_string(), "value3".to_string());

    c.bench_function("comparison_hashmap_serde_qs_serialize", |b| {
        b.iter(|| {
            serde_qs::to_string(black_box(&data)).unwrap()
        })
    });
}

fn comparison_hashmap_serde_urlencoded(c: &mut Criterion) {
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());
    data.insert("key3".to_string(), "value3".to_string());

    c.bench_function("comparison_hashmap_serde_urlencoded_serialize", |b| {
        b.iter(|| {
            serde_urlencoded::to_string(black_box(&data)).unwrap()
        })
    });
}

// Deserialization comparisons
fn comparison_simple_struct_deserialize_serde_qs(c: &mut Criterion) {
    let query = "id=42&name=test_user&active=true";

    c.bench_function("comparison_simple_struct_serde_qs_deserialize", |b| {
        b.iter(|| {
            let _: SimpleStruct = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn comparison_simple_struct_deserialize_serde_urlencoded(c: &mut Criterion) {
    let query = "id=42&name=test_user&active=true";

    c.bench_function("comparison_simple_struct_serde_urlencoded_deserialize", |b| {
        b.iter(|| {
            let _: SimpleStruct = serde_urlencoded::from_str(black_box(query)).unwrap();
        })
    });
}

fn comparison_hashmap_deserialize_serde_qs(c: &mut Criterion) {
    let query = "key1=value1&key2=value2&key3=value3";

    c.bench_function("comparison_hashmap_serde_qs_deserialize", |b| {
        b.iter(|| {
            let _: HashMap<String, String> = serde_qs::from_str(black_box(query)).unwrap();
        })
    });
}

fn comparison_hashmap_deserialize_serde_urlencoded(c: &mut Criterion) {
    let query = "key1=value1&key2=value2&key3=value3";

    c.bench_function("comparison_hashmap_serde_urlencoded_deserialize", |b| {
        b.iter(|| {
            let _: HashMap<String, String> = serde_urlencoded::from_str(black_box(query)).unwrap();
        })
    });
}

criterion_group!(
    serialize_simple,
    serialize_simple_struct,
    serialize_simple_with_option_some,
    serialize_simple_with_option_none,
    serialize_simple_vec,
    serialize_simple_hashmap
);

criterion_group!(
    serialize_complex,
    serialize_nested_struct,
    serialize_deep_nested,
    serialize_large_vec
);

criterion_group!(
    deserialize_simple,
    deserialize_simple_struct,
    deserialize_simple_with_option_some,
    deserialize_simple_with_option_none,
    deserialize_simple_vec,
    deserialize_simple_hashmap
);

criterion_group!(
    deserialize_complex,
    deserialize_nested_struct,
    deserialize_deep_nested,
    deserialize_large_vec
);

criterion_group!(
    comparison,
    comparison_simple_struct_serde_qs,
    comparison_simple_struct_serde_urlencoded,
    comparison_hashmap_serde_qs,
    comparison_hashmap_serde_urlencoded,
    comparison_simple_struct_deserialize_serde_qs,
    comparison_simple_struct_deserialize_serde_urlencoded,
    comparison_hashmap_deserialize_serde_qs,
    comparison_hashmap_deserialize_serde_urlencoded
);

criterion_main!(serialize_simple, serialize_complex, deserialize_simple, deserialize_complex, comparison);