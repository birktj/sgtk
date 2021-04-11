use rand::prelude::*;
use crate::graph::Graph;
use crate::permutation::Permutation;

pub fn graph<G: Graph>(n: usize) -> G {
    let mut graph = G::empty();
    for i in 0..n {
        graph.add_node(i);
    }

    let edge_ratio = rand::thread_rng().gen_range(0.0..=1.0);
    //let edge_ratio = rand::thread_rng().gen_range(0.7..0.9);
    let edge_count = ((n*(n+1) / 2) as f32 * edge_ratio) as usize;

    for _ in 0..edge_count {
        let u = rand::thread_rng().gen_range(1..n);
        let v = rand::thread_rng().gen_range(0..u);
        graph.add_edge(u, v);
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

