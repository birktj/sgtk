use sgtk::graph::{subgraphs, Graph32};
use sgtk::bitset::Bitset32;
use sgtk::prelude::*;
use std::collections::{HashSet, HashMap};
use crate::is_obstruction;

fn search_obstruction<G: Graph>(mut graph: G, u: usize, mut edges: G::Set) -> Option<G> {
    if let Some(v) = edges.smallest() {
        edges.clear(v);
        if let Some(obs) = search_obstruction(graph.clone(), u, edges.clone()) {
            return Some(obs)
        }
        graph.del_edge(u, v);
        if sgtk::toroidal::find_embedding(&graph).is_some() {
            return None
        } else if let Some(obs) = search_obstruction(graph.clone(), u, edges) {
            return Some(obs)
        }
        None
    } else if is_obstruction(&graph) {
        Some(graph)
    } else {
        None
    }
}

fn find_obstruction<G: Graph>(mut graph: G) -> G {
    for (u, v) in graph.clone().edges() {
        graph.del_edge(u, v);
        if sgtk::toroidal::find_embedding(&graph).is_some() {
            graph.add_edge(u, v);
        }
    }

    for u in graph.nodes().iter() {
        if graph.siblings(u).count() == 2 {
            let mut s = graph.siblings(u);
            graph.del_node(u);
            let v = s.smallest().unwrap();
            s.clear(v);
            let w = s.smallest().unwrap();
            graph.add_edge(v, w);
        }
    }

    for u in graph.nodes().iter() {
        let s = graph.siblings(u);
        graph.del_node(u);
        if sgtk::toroidal::find_embedding(&graph).is_some() {
            graph.add_node(u);
            graph.add_edges(u, &s);
        }
    }

    graph
}

/*
struct ObstructionSearcher<G> {
    orig: G,
    obstructions: Vec<G>,
    uncovered_obstructions: Vec<G>,
}

impl<G: Graph> ObstructionSearcher<G> {
    fn search(&mut self, mut graph: G) -> bool {
        if let Some(obs) = self.uncovered_obstructions.pop() {
            let mut found = false;
            for (u, v) in obs.edges() {
                if !graph.has_edge(u, v) {
                    continue
                }
                graph.del_edge(u, v);
                //if sgtk::toroidal::find_embedding(&graph).is_none() {
                    let uncovered = std::mem::replace(&mut self.uncovered_obstructions, Vec::new());
                    let (covered, uncovered) = uncovered.into_iter()
                        .partition(|obs| obs.has_edge(u, v));
                    self.uncovered_obstructions = uncovered;

                    if self.search(graph.clone()) {
                        found = true;
                    }
                    self.uncovered_obstructions.extend(covered);
                //}
                graph.add_edge(u, v);
            }
            self.uncovered_obstructions.push(obs);
            found
        } else if sgtk::toroidal::find_embedding(&graph).is_none() {
            for (u, v) in graph.clone().edges() {
                graph.del_edge(u, v);
                if sgtk::toroidal::find_embedding(&graph).is_some() {
                    graph.add_edge(u, v);
                }
            }

            for u in graph.nodes().iter() {
                let s = graph.siblings(u);
                graph.del_node(u);
                if sgtk::toroidal::find_embedding(&graph).is_some() {
                    graph.add_node(u);
                    graph.add_edges(u, &s);
                }
            }
            dbg!(sgtk::parse::to_graph6(&graph));
            self.obstructions.push(graph.clone());
            self.uncovered_obstructions.push(graph);
            true
        } else {
            false
        }
    }
}

pub fn find_all_obstructions<G: Graph>(graph: G) -> Vec<G> {
    let mut searcher = ObstructionSearcher {
        orig: graph.clone(),
        obstructions: Vec::new(),
        uncovered_obstructions: Vec::new(),
    };

    while searcher.search(graph.clone()) {}

    searcher.obstructions.into_iter()
        .map(|mut obs| {
            for u in obs.nodes().iter() {
                if obs.siblings(u).count() < 2 {
                    obs.del_node(u);
                } else if obs.siblings(u).count() == 2 {
                    let mut s = obs.siblings(u);
                    obs.del_node(u);
                    let v = s.smallest().unwrap();
                    s.clear(v);
                    let w = s.smallest().unwrap();
                    obs.add_edge(v, w);
                }
            }
            obs
        })
        .collect()
}
*/


pub fn find_splitdel_min(graph: &sgtk::graph::Graph32) -> Option<sgtk::graph::Graph32> {
    for (u, v) in graph.edges() {
        let mut graph = graph.clone();
        graph.contract_edge(u, v);
        let mut new_edges = graph.siblings(u).invert().intersection(&graph.nodes());
        new_edges.clear(u);
        graph.add_edges(u, &new_edges);
        if sgtk::toroidal::find_embedding(&graph).is_some() {
            continue
        }
        if let Some(obs) = search_obstruction(graph, u, new_edges) {
            return Some(obs)
        }
            /*
        for v in new_edges.iter() {
            graph.del_edge(u, v);
            if sgtk::toroidal::find_embedding(&graph).is_some() {
                graph.add_edge(u, v);
            }
        }
        if is_obstruction(&graph) {
            */
    }

    None
}

pub fn gen_splitdel<'a>(graph: &'a Graph32) -> impl 'a + Iterator<Item = Graph32> {
    let v = graph.nodes().invert().smallest().unwrap();
    /*
    graph.nodes().iter().flat_map(move |u| {
        let mut g = graph.clone();
        g.add_node(v);
        let s = g.siblings(u);
        g.add_edge(u, v);
        g.add_edges(v, &s);

        /*
        for (u, v) in g.clone().edges() {
            g.del_edge(u, v);
            if sgtk::toroidal::find_embedding(&g).is_some() {
                g.add_edge(u, v);
            }
        }
        if is_obstruction(&g) {
            Some(g)
        } else {
            None
        }
        */

        let mut searcher = SplitDelSearcher { found: Vec::new() };
        //searcher.search_obstruction(g.clone(), g.edges());
        searcher.splitdel_search(g.clone(), u, v, s.clone());
        searcher.found.into_iter()
        //find_all_obstructions(g)
    })
    */
    let mut searcher = SplitDelSearcher2::new();
    //let orbits = compute_orbits(graph.clone());
    for u in graph.nodes().iter() {
        /*
        if orbits[u] != u {
            continue
        }
        */
        let mut g = graph.clone();
        g.add_node(v);
        let s = g.siblings(u);
        g.add_edge(u, v);
        searcher.split_search(g, u, v, s);
    }
    searcher.found.into_iter()
    /*
    let mut searcher = ObstructionSearcher::new();
    let mut g = graph.clone();
    g.add_node(v);
    g.add_edges(v, &graph.nodes());
    searcher.search(g);
    searcher.found.into_iter()
    */
}

struct SplitDelSearcher<G> {
    found: Vec<G>
}

impl<G: Graph + Ord> SplitDelSearcher<G> {
    fn search_obstruction<'a>(&mut self, mut graph: G, mut edge_iter: sgtk::graph::EdgeIter<'a, G>) {
        for u in graph.nodes().iter() {
            if graph.siblings(u).count() < 3 {
                return
            }
        }
        if let Some((u, v)) = edge_iter.next() {
            self.search_obstruction(graph.clone(), edge_iter.clone());
            graph.del_edge(u, v);
            if sgtk::toroidal::find_embedding(&graph).is_none() {
                self.search_obstruction(graph, edge_iter);
            }
        } else {
            if is_obstruction(&graph) {
                self.found.push(graph);
            }
        }
    }

    fn splitdel_search(&mut self, mut graph: G, u: usize, v: usize, mut edges: G::Set) {
        if let Some(w) = edges.smallest() {
            let mut candidate = true;
            edges.clear(w);
            //self.splitdel_search(graph.clone(), u, v, edges.clone());
            graph.del_edge(u, w);
            if sgtk::toroidal::find_embedding(&graph).is_none() {
                self.splitdel_search(graph.clone(), u, v, edges.clone());
                candidate = false;
            }
            graph.del_edge(v, w);
            if sgtk::toroidal::find_embedding(&graph).is_none() {
                self.splitdel_search(graph.clone(), u, v, edges.clone());
                candidate = false;
            }
            graph.add_edge(u, w);
            if sgtk::toroidal::find_embedding(&graph).is_none() {
                self.splitdel_search(graph.clone(), u, v, edges.clone());
                candidate = false;
            }

            graph.add_edge(v, w);
            if candidate { // && is_obstruction(&graph) {
                //self.found.push(graph);
                self.found.push(find_obstruction(graph));
            }
        } else {
            self.found.push(find_obstruction(graph));
            /*
            if is_obstruction(&graph) {
                self.found.push(graph);
            }
            */
        }
    }
}

struct SplitDelSearcher2 {
    found: Vec<Graph32>,
    visited: HashSet<Graph32>,
    mem: HashMap<Graph32, bool>,
}

impl SplitDelSearcher2 {
    fn new() -> Self {
        Self {
            found: Vec::new(),
            visited: HashSet::new(),
            mem: HashMap::new(),
        }
    }

    fn split_search(&mut self, mut graph: Graph32, u: usize, v: usize, mut es: Bitset32) {
        if let Some(w) = es.smallest() {
            es.clear(w);
            self.split_search(graph.clone(), u, v, es.clone());
            graph.del_edge(u, w);
            graph.add_edge(v, w);
            self.split_search(graph.clone(), u, v, es.clone());
        } else {
            self.del_search(graph);
        }
    }

    fn is_toroidal(&mut self, graph: &Graph32) -> bool {
        if let Some(r) = self.mem.get(graph) {
            *r
        } else {
            let r = sgtk::toroidal::find_embedding(graph).is_some();
            self.mem.insert(graph.clone(), r);
            r
        }
    }

    fn del_search(&mut self, graph: Graph32) {
        let graph = graph.to_canonical();
        if self.visited.contains(&graph) {
            return
        }
        self.visited.insert(graph.clone());

        for u in graph.nodes().iter() {
            if graph.siblings(u).count() < 3 {
                return
            }
        }

        let mut obstruction = true;

        for (u, v) in graph.edges() {
            let mut graph = graph.clone();
            graph.del_edge(u, v);
            if !self.is_toroidal(&graph) {
                obstruction = false;
                self.del_search(graph.clone());
            }
        }

        if obstruction {
            let mut graph = graph;
            for u in graph.nodes() {
                /*
                if graph.siblings(u).count() < 2 {
                    graph.del_node(u);
                } else if graph.siblings(u).count() == 2 {
                    let mut s = graph.siblings(u);
                    graph.del_node(u);
                    let v = s.smallest().unwrap();
                    s.clear(v);
                    let w = s.smallest().unwrap();
                    graph.add_edge(v, w);
                }
                */
            }
            self.found.push(graph);
        }
    }
}

fn compute_orbits(graph: Graph32) -> [usize; 32] {
    let n = graph.nodes().count();
    let auto_gens = sgtk::iso::search_tree(graph).automorphisms;

    let mut orbits = [0; 32];
    for i in 0..n {
        orbits[i] = i;
    }

    for perm in auto_gens {
        for i in 0..n {
            if perm.get(i) != i {
                let mut j1 = usize::from(orbits[i]);
                while usize::from(orbits[j1]) != j1 {
                    j1 = usize::from(orbits[j1]);
                }

                let mut j2 = usize::from(orbits[perm.get(i)]);
                while usize::from(orbits[j2]) != j2 {
                    j2 = usize::from(orbits[j2]);
                }

                if j1 < j2 {
                    orbits[j2] = j1;
                } else {
                    orbits[j1] = j2;
                }
            }
        }

        for i in 0..n {
            orbits[i] = orbits[orbits[i]];
        }
    }

    //dbg!(&orbits);

    orbits
}

fn compute_autos(graph: Graph32) -> HashSet<Graph32> {
    let auto_gens = sgtk::iso::search_tree(graph).automorphisms;

    let mut autos = HashSet::new();

    let mut queue = Vec::new();
    queue.push(graph);

    while let Some(graph) = queue.pop() {
        for perm in &auto_gens {
            let mut g = graph.clone();
            g.shuffle(perm);
            if !autos.contains(&g) {
                queue.push(g.clone());
                autos.insert(g);
            }
        }
    }

    autos
}

pub struct ObstructionSearcher {
    pub found: Vec<Graph32>,
    visited: HashSet<Graph32>,
    mem: HashMap<Graph32, bool>,
}

impl ObstructionSearcher {
    pub fn new() -> Self {
        Self {
            found: Vec::new(),
            visited: HashSet::new(),
            mem: HashMap::new(),
        }
    }

    fn is_toroidal(&mut self, graph: &Graph32) -> bool {
        if let Some(r) = self.mem.get(graph) {
            *r
        } else {
            let r = sgtk::toroidal::find_embedding(graph).is_some();
            self.mem.insert(graph.clone(), r);
            r
        }
    }

    pub fn search(&mut self, graph: Graph32) {
        self.search_internal(graph.to_canonical());
    }

    fn search_internal(&mut self, graph: Graph32) {
        if self.visited.contains(&graph) {
            return
        }
        self.visited.insert(graph.clone());

        for u in graph.nodes().iter() {
            if graph.siblings(u).count() < 3 {
                return
            }
        }

        let mut obstruction = true;

        for (u, v) in graph.edges() {
            let mut graph = graph.clone();
            graph.del_edge(u, v);
            let graph = graph.to_canonical();
            if !self.is_toroidal(&graph) {
                obstruction = false;
                self.search_internal(graph);
            }
        }

        if obstruction {
            /*
            let mut graph = graph;
            for u in graph.nodes() {
                if graph.siblings(u).count() < 2 {
                    graph.del_node(u);
                } else if graph.siblings(u).count() == 2 {
                    let mut s = graph.siblings(u);
                    graph.del_node(u);
                    let v = s.smallest().unwrap();
                    s.clear(v);
                    let w = s.smallest().unwrap();
                    graph.add_edge(v, w);
                }
            }
            dbg!(sgtk::parse::to_graph6(&graph));
            */
            self.found.push(graph.to_canonical());
        }
    }
}
