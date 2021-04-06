use std::collections::HashSet;
use crate::Graph16;
use crate::bitset::{Bitset, Bitset16};
use crate::iso::SearchTree;

pub struct Enumerator16 {
    maxn: usize,
    //pub graphs: Vec<Graph16>,
    pub graphs: HashSet<Graph16>,
    smaller_graphs: HashSet<Graph16>,
    pub auto_prune: bool,
    pub canonical_count: u32,
}

impl Enumerator16 {
    pub fn new(maxn: usize) -> Enumerator16 {
        Enumerator16 {
            maxn,
            //graphs: Vec::new(),
            graphs: HashSet::new(),
            smaller_graphs: HashSet::new(),
            auto_prune: true,
            canonical_count: 0,
        }
    }

    fn enumerate_inner(&mut self, mut graph: Graph16, n: usize) {
        if n >= self.maxn {
            return
        }

        graph.add_node(n);

        for edges in Bitset16::enumerate(n) {
            let mut orig_graph = graph;
            orig_graph.add_edges(n, edges); //.to_canonical();
            
            let mut search_tree = SearchTree::new(orig_graph);
            search_tree.auto_prune = self.auto_prune;
            let graph = search_tree.find_canonical();
            self.canonical_count += 1;

            //assert_eq!(graph, graph.to_canonical(), "original: {:?}", orig_graph);
            
            if n + 1 == self.maxn {
                self.graphs.insert(graph);
            } else if !self.smaller_graphs.contains(&graph) {
                self.smaller_graphs.insert(graph);
                self.enumerate_inner(graph, n+1);
            }
        }
    }

    pub fn enumerate(&mut self) {
        let graph = Graph16::new(1);
        self.enumerate_inner(graph, 1);
    }
}
