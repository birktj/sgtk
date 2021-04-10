#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use sgtk::graph::{Graph, Graph16};

pub fn benchmark(c: &mut Criterion) {
    c.bench_function("k5_to_canonical", |b| b.iter(|| {
        black_box(Graph16::complete(5)).to_canonical()
    }));

    c.bench_function("k10_to_canonical", |b| b.iter(|| {
        black_box(Graph16::complete(10)).to_canonical()
    }));

    c.bench_function("k16_to_canonical", |b| b.iter(|| {
        black_box(Graph16::complete(16)).to_canonical()
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
