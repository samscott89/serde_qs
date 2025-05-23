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


## Benchmark Results

### Performance Baseline (v1.0)

#### Simple Data Structures
| Operation | Structure | Time |
|-----------|-----------|------|
| Serialize | Simple struct | ~183ns |
| Deserialize | Simple struct | ~296ns |
| Serialize | HashMap (3 items) | ~199ns |
| Deserialize | HashMap (3 items) | ~455ns |
| Serialize | Vec (5 items) | ~309ns |
| Deserialize | Vec (5 items) | ~491ns |

#### Complex/Nested Data Structures
| Operation | Structure | Time |
|-----------|-----------|------|
| Serialize | Nested struct (2 levels) | ~739ns |
| Deserialize | Nested struct (2 levels) | ~1.28μs |
| Serialize | Deep nested (4 levels) | ~1.10μs |
| Deserialize | Deep nested (4 levels) | ~2.47μs |
| Serialize | Large vec (100 items) | ~4.97μs |
| Deserialize | Large vec (100 items) | ~8.16μs |

#### Comparison with serde_urlencoded
| Structure | Library | Serialize | Deserialize |
|-----------|---------|-----------|-------------|
| Simple struct | serde_qs | ~181ns | ~295ns |
| Simple struct | serde_urlencoded | ~136ns (**33% faster**) | ~183ns (**61% faster**) |
| HashMap | serde_qs | ~198ns | ~464ns |
| HashMap | serde_urlencoded | ~174ns (**14% faster**) | ~352ns (**32% faster**) |
