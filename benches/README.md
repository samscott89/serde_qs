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
| Serialize | Simple struct | ~419ns |
| Deserialize | Simple struct | ~294ns |
| Serialize | HashMap (3 items) | ~534ns |
| Deserialize | HashMap (3 items) | ~462ns |
| Serialize | Vec (5 items) | ~1.13μs |
| Deserialize | Vec (5 items) | ~496ns |

#### Complex/Nested Data Structures
| Operation | Structure | Time |
|-----------|-----------|------|
| Serialize | Nested struct (2 levels) | ~2.14μs |
| Deserialize | Nested struct (2 levels) | ~1.36μs |
| Serialize | Deep nested (4 levels) | ~3.69μs |
| Deserialize | Deep nested (4 levels) | ~2.57μs |
| Serialize | Large vec (100 items) | ~21.5μs |
| Deserialize | Large vec (100 items) | ~8.76μs |

#### Comparison with serde_urlencoded
| Structure | Library | Serialize | Deserialize |
|-----------|---------|-----------|-------------|
| Simple struct | serde_qs | ~424ns | ~297ns |
| Simple struct | serde_urlencoded | ~135ns (**3.1x faster**) | ~172ns (**1.7x faster**) |
| HashMap | serde_qs | ~514ns | ~458ns |
| HashMap | serde_urlencoded | ~178ns (**2.9x faster**) | ~345ns (**1.3x faster**) |
