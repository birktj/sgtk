#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};

pub fn benchmark(c: &mut Criterion) {
    let graph: sgtk::graph::Graph16 = sgtk::parse::from_graph6("F@|ZO");
    c.bench_function("planar_test", |b| b.iter(|| {
        sgtk::planar::fastdmp(&black_box(graph))
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
