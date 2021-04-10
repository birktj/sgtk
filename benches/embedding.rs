#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use sgtk::graph::{Graph, Graph16};
use sgtk::embedding::RotationSystem16;

pub fn benchmark(c: &mut Criterion) {
    let mut k33 = Graph16::empty();
    for i in 0..6 {
        k33.add_node(i);
    }
    for i in 0..3 {
        for j in 3..6 {
            k33.add_edge(i, j);
        }
    }

    c.bench_function("count_k5_faces", |b| b.iter(|| {
        RotationSystem16::simple(&black_box(Graph16::complete(5)))
            .faces().count()
    }));

    c.bench_function("count_k33_faces", |b| b.iter(|| {
        RotationSystem16::simple(&black_box(k33))
            .faces().count()
    }));

    c.bench_function("enumerate_k5_embeddings", |b| b.iter(|| {
        RotationSystem16::enumerate(&black_box(Graph16::complete(5))).count()
    }));

    c.bench_function("enumerate_k33_embeddings", |b| b.iter(|| {
        RotationSystem16::enumerate(&black_box(k33)).count()
    }));

    c.bench_function("enumerate_k5_toroidal_embeddings", |b| b.iter(|| {
        RotationSystem16::enumerate(&black_box(Graph16::complete(5)))
            .filter(|embedding| embedding.genus() == 1)
            .count()
    }));

    c.bench_function("enumerate_k33_toroidal_embeddings", |b| b.iter(|| {
        RotationSystem16::enumerate(&black_box(k33))
            .filter(|embedding| embedding.genus() == 1)
            .count()
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
