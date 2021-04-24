#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use sgtk::graph::{minors, subgraphs, Graph, Graph16};

pub fn benchmark(c: &mut Criterion) {
    c.bench_function("toroidal_k7", |b| b.iter(|| {
        sgtk::toroidal::find_embedding(&black_box(Graph16::complete(7))).is_some()
    }));

    let obstruction: Graph16 = 
        sgtk::parse::from_upper_tri("9 111000011100001100001000011111111111")
        .unwrap();
    c.bench_function("toroidal_obstruction", |b| b.iter(|| {
        sgtk::toroidal::find_embedding(&black_box(obstruction)).is_some()
    }));
    c.bench_function("toroidal_obstruction_check_minors", |b| b.iter(|| {
        minors(&black_box(obstruction))
            .filter(|minor| minor.is_connected())
            .filter(|minor| sgtk::toroidal::find_embedding(minor).is_some())
            .count()
    }));

    let obstruction: Graph16 =
        sgtk::parse::from_upper_tri("15 111000000000000000000110000001010000000101000000000001000001001000100100000100000001000010001100101111000")
        .unwrap();

    c.bench_function("toroidal_obstruction_15_check_subgraphs", |b| b.iter(|| {
        subgraphs(&black_box(obstruction))
            .filter(|minor| sgtk::toroidal::find_embedding(minor).is_some())
            .count()
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
