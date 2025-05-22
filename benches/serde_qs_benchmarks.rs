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

criterion_group!(
    serialize_simple,
    serialize_simple_struct,
    serialize_simple_with_option_some,
    serialize_simple_with_option_none,
    serialize_simple_vec,
    serialize_simple_hashmap
);

criterion_group!(
    deserialize_simple,
    deserialize_simple_struct,
    deserialize_simple_with_option_some,
    deserialize_simple_with_option_none,
    deserialize_simple_vec,
    deserialize_simple_hashmap
);

criterion_main!(serialize_simple, deserialize_simple);