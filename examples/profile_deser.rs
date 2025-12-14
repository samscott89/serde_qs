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
    println!("Profiling deserialization for {:?}...", profile_duration);

    match std::env::args().nth(1).as_deref() {
        Some("simple") => profile_simple_timed(profile_duration),
        Some("complex") => profile_complex_timed(profile_duration),
        Some("mixed") => profile_mixed_workload(profile_duration),
        _ => {
            println!("Usage: cargo run --release --example profile_deser [simple|complex|mixed]");
            println!("Defaulting to mixed workload...");
            profile_mixed_workload(profile_duration);
        }
    }
}

fn profile_simple_timed(duration: Duration) {
    println!("Running simple deserialization profile...");
    let query = "id=42&name=test_user&active=true";

    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed() < duration {
        // Run in batches to reduce timing overhead
        for _ in 0..1000 {
            let _: SimpleStruct = serde_qs::from_str(black_box(query)).unwrap();
            count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("Completed {} deserializations in {:?}", count, elapsed);
    println!("Average: {:?} per operation", elapsed / count as u32);
}

fn profile_complex_timed(duration: Duration) {
    println!("Running complex deserialization profile...");
    let query = "id=42&name=Acme&phone=12345&address[city]=Carrot+City&\
                 address[street]=Special-Street*+No.+11&address[postcode]=12345&\
                 user_ids[0]=1&user_ids[1]=2&user_ids[2]=3&user_ids[3]=4";

    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed() < duration {
        for _ in 0..1000 {
            let _: QueryParams = serde_qs::from_str(black_box(query)).unwrap();
            count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("Completed {} deserializations in {:?}", count, elapsed);
    println!("Average: {:?} per operation", elapsed / count as u32);
}

fn profile_mixed_workload(duration: Duration) {
    println!("Running mixed workload profile...");

    // Different query patterns to profile
    let simple_query = "id=42&name=test_user&active=true";
    let map_query = "key1=value1&key2=value2&key3=value3&key4=value4&key5=value5";
    let complex_query = "id=42&name=Acme&phone=12345&address[city]=Carrot+City&\
                         address[street]=Special-Street*+No.+11&address[postcode]=12345&\
                         user_ids[0]=1&user_ids[1]=2&user_ids[2]=3&user_ids[3]=4";

    let start = Instant::now();
    let mut simple_count = 0u64;
    let mut map_count = 0u64;
    let mut complex_count = 0u64;

    while start.elapsed() < duration {
        // Mix different deserialization patterns
        for _ in 0..100 {
            let _: SimpleStruct = serde_qs::from_str(black_box(simple_query)).unwrap();
            simple_count += 1;
        }

        for _ in 0..100 {
            let _: HashMap<String, String> = serde_qs::from_str(black_box(map_query)).unwrap();
            map_count += 1;
        }

        for _ in 0..100 {
            let _: QueryParams = serde_qs::from_str(black_box(complex_query)).unwrap();
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
