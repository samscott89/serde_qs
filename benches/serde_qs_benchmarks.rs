use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_placeholder(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // Placeholder benchmark to verify setup works
            1 + 1
        })
    });
}

criterion_group!(benches, benchmark_placeholder);
criterion_main!(benches);