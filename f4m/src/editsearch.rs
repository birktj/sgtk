use sgtk::graph::{subgraphs, Graph32};
use sgtk::bitset::Bitset32;
use sgtk::prelude::*;
use std::collections::{HashSet, HashMap};

pub struct EditSearcher {
    pub visited: HashSet<Graph32>,
    maxlen: usize,
}

impl EditSearcher {
    pub fn new(maxlen: usize) -> EditSearcher {
        EditSearcher {
            visited: HashSet::new(),
            maxlen
        }
    }

    pub fn search(&mut self, graph: Graph32, len: usize) {
        let graph = graph.to_canonical();
        if self.visited.contains(&graph) {
            return
        }
        self.visited.insert(graph.clone());
        if len >= self.maxlen {
            return
        }

        for u in graph.nodes().iter() {
            for v in graph.nodes().intersection(&Bitset32::mask_ge(u)).iter() {
                let mut graph = graph.clone();
                if !graph.has_edge(u, v) {
                    graph.add_edge(u, v);
                    self.search(graph, len+1);
                }
            }
        }
    }
}
