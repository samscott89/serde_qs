# serde_qs Benchmarks

This directory contains performance benchmarks for the `serde_qs` crate using [Criterion.rs](https://github.com/bheisler/criterion.rs).

## Overview

The benchmarks are organized into several categories to measure performance across different use cases:

### 1. Simple Data Structure Benchmarks
- **Flat structs**: Basic structures with primitive types
- **Optional fields**: Structures with `Option<T>` fields  
- **Collections**: `Vec<T>` and `HashMap<K,V>` performance
- **Primitive types**: String, integer, boolean serialization/deserialization

### 2. Complex/Nested Data Structure Benchmarks
- **Nested structs**: Objects containing other objects (Address within QueryParams)
- **Deep nesting**: 3-4 levels of nested structures
- **Large collections**: Arrays with 100+ items
- **Mixed structures**: Combinations of objects, arrays, and maps

### 3. Performance Comparisons
- **vs serde_urlencoded**: Direct comparison for flat structures where both libraries work
- **Scaling analysis**: Performance characteristics as data complexity increases


## Benchmark Results

### Performance Baseline (v1.0)

#### Simple Data Structures
| Operation | Structure | Time |
|-----------|-----------|------|
| Serialize | Simple struct | ~425ns |
| Deserialize | Simple struct | ~362ns |
| Serialize | HashMap (3 items) | ~525ns |
| Deserialize | HashMap (3 items) | ~493ns |
| Serialize | Vec (5 items) | ~1.1μs |
| Deserialize | Vec (5 items) | ~925ns |

#### Complex/Nested Data Structures
| Operation | Structure | Time |
|-----------|-----------|------|
| Serialize | Nested struct (2 levels) | ~2.1μs |
| Deserialize | Nested struct (2 levels) | ~1.8μs |
| Serialize | Deep nested (4 levels) | ~3.7μs |
| Deserialize | Deep nested (4 levels) | ~3.6μs |
| Serialize | Large vec (100 items) | ~21μs |
| Deserialize | Large vec (100 items) | ~21μs |

#### Comparison with serde_urlencoded
| Structure | Library | Serialize | Deserialize |
|-----------|---------|-----------|-------------|
| Simple struct | serde_qs | ~425ns | ~384ns |
| Simple struct | serde_urlencoded | ~134ns (**3x faster**) | ~166ns (**2.3x faster**) |
| HashMap | serde_qs | ~536ns | ~560ns |
| HashMap | serde_urlencoded | ~175ns (**3x faster**) | ~352ns (**1.6x faster**) |
