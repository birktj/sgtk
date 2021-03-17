use rand::prelude::*;
use crate::Graph16;
use crate::seq::Seq16;

pub fn graph16(n: usize) -> Graph16 {
    let mut graph = Graph16::new(n);

    let edge_ratio = rand::thread_rng().gen_range(0.0..=1.0);
    let edge_count = ((n*(n+1) / 2) as f32 * edge_ratio) as usize;

    for _ in 0..edge_count {
        let u = rand::thread_rng().gen_range(0..n);
        let v = rand::thread_rng().gen_range(0..n);
        graph = graph.add_edge(u, v);
    }

    graph
}

pub fn permutation(n: usize) -> Seq16 {
    let mut perm = [0; 16];
    for i in 0..16 {
        perm[i] = i as u8;
    }

    let mut rng = thread_rng();
    perm[0..n].shuffle(&mut rng);

    Seq16::from_slice(&perm as &[_])
}

