use rand::prelude::*;
use crate::graph::Graph;
use crate::permutation::Permutation;

pub fn graph<G: Graph>(n: usize) -> G {
    let mut graph = G::empty();
    for i in 0..n {
        graph.add_node(i);
    }

    let edge_ratio = 0.5; // rand::thread_rng().gen_range(0.0..=1.0);

    for u in 0..n {
        for v in 0..u {
            if rand::thread_rng().gen_bool(edge_ratio) {
                graph.add_edge(u, v);
            }
        }
    }

    graph
}

pub fn permutation<P: Permutation>(n: usize) -> P {
    let mut perm = vec![0; n];
    for i in 0..n {
        perm[i] = i;
    }

    let mut rng = thread_rng();
    perm[0..n].shuffle(&mut rng);

    P::from_iter(perm.into_iter().enumerate()).unwrap()
}

