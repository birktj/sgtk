use sgtk::*;
use sgtk::graph::{Graph, Graph16};

fn test_canon_random_graph(n: usize) {
    test_canon_random_perm(n, random::graph(n));
}

fn test_canon_random_perm(n: usize, graph1: Graph16) {
    let perm = random::permutation(n);
    let mut graph2 = graph1.clone();
    graph2.shuffle(&perm);

    let canon1 = graph1.to_canonical();
    let canon2 = graph2.to_canonical();

    assert_eq!(canon1, canon2);
    assert!(canon1.is_canonical());
}

#[test]
fn random_graph5() {
    for _i in 0..1000 {
        test_canon_random_graph(5);
    }
}

#[test]
fn random_graph10() {
    for _i in 0..10 {
        test_canon_random_graph(10);
    }
}

#[test]
fn random_graph16() {
    for _i in 0..10 {
        test_canon_random_graph(16);
    }
}

#[test]
fn regular_graphs() {
    test_canon_random_perm(5, Graph16::complete(5));
    test_canon_random_perm(10, Graph16::complete(10));
    test_canon_random_perm(16, Graph16::complete(16));
}
