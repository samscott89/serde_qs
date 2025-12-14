use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleStruct {
    id: u32,
    name: String,
    active: bool,
}

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

fn main() {
    let profile_duration = Duration::from_secs(30); // Run for 30 seconds
    println!("Profiling serialization for {:?}...", profile_duration);

    match std::env::args().nth(1).as_deref() {
        Some("simple") => profile_simple_timed(profile_duration),
        Some("complex") => profile_complex_timed(profile_duration),
        Some("mixed") => profile_mixed_workload(profile_duration),
        _ => {
            println!("Usage: cargo run --release --example profile_ser [simple|complex|mixed]");
            println!("Defaulting to mixed workload...");
            profile_mixed_workload(profile_duration);
        }
    }
}

fn profile_simple_timed(duration: Duration) {
    println!("Running simple serialization profile...");
    let data = SimpleStruct {
        id: 42,
        name: "test_user".to_string(),
        active: true,
    };

    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed() < duration {
        // Run in batches to reduce timing overhead
        for _ in 0..1000 {
            let _ = serde_qs::to_string(black_box(&data)).unwrap();
            count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("Completed {} serializations in {:?}", count, elapsed);
    println!("Average: {:?} per operation", elapsed / count as u32);
}

fn profile_complex_timed(duration: Duration) {
    println!("Running complex serialization profile...");
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

    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed() < duration {
        for _ in 0..1000 {
            let _ = serde_qs::to_string(black_box(&data)).unwrap();
            count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("Completed {} serializations in {:?}", count, elapsed);
    println!("Average: {:?} per operation", elapsed / count as u32);
}

fn profile_mixed_workload(duration: Duration) {
    println!("Running mixed workload profile...");

    // Different data patterns to profile
    let simple_data = SimpleStruct {
        id: 42,
        name: "test_user".to_string(),
        active: true,
    };

    let mut map_data = HashMap::new();
    map_data.insert("key1".to_string(), "value1".to_string());
    map_data.insert("key2".to_string(), "value2".to_string());
    map_data.insert("key3".to_string(), "value3".to_string());
    map_data.insert("key4".to_string(), "value4".to_string());
    map_data.insert("key5".to_string(), "value5".to_string());

    let complex_data = QueryParams {
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

    let start = Instant::now();
    let mut simple_count = 0u64;
    let mut map_count = 0u64;
    let mut complex_count = 0u64;

    while start.elapsed() < duration {
        // Mix different serialization patterns
        for _ in 0..100 {
            let _ = serde_qs::to_string(black_box(&simple_data)).unwrap();
            simple_count += 1;
        }

        for _ in 0..100 {
            let _ = serde_qs::to_string(black_box(&map_data)).unwrap();
            map_count += 1;
        }

        for _ in 0..100 {
            let _ = serde_qs::to_string(black_box(&complex_data)).unwrap();
            complex_count += 1;
        }
    }

    let elapsed = start.elapsed();
    let total = simple_count + map_count + complex_count;

    println!("\nProfile complete:");
    println!("Total operations: {}", total);
    println!("- Simple: {}", simple_count);
    println!("- HashMap: {}", map_count);
    println!("- Complex: {}", complex_count);
    println!("Total time: {:?}", elapsed);
    println!("Average: {:?} per operation", elapsed / total as u32);
}
