use sgtk::graph::subgraphs;
use sgtk::prelude::*;

fn is_obstruction<G: Graph>(graph: &G) -> bool {
    if sgtk::toroidal::find_embedding(graph).is_some() {
        return false
    }
    for subgraph in subgraphs(graph) {
        if sgtk::toroidal::find_embedding(&subgraph).is_none() {
            return false
        }
    }

    true
}

fn is_minor_obstruction<G: Graph>(graph: &G) -> bool {
    for (u, v) in graph.edges() {
        let mut graph = graph.to_owned();
        graph.contract_edge(u, v);
        if sgtk::toroidal::find_embedding(&graph).is_none() {
            return false
        }
    }

    true
}

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
        let s = graph.siblings(u);
        graph.del_node(u);
        if sgtk::toroidal::find_embedding(&graph).is_some() {
            graph.add_node(u);
            graph.add_edges(u, &s);
        }
    }

    graph
}

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
    }

    None
}

pub fn gen_splitdel<'a, G: Graph + Ord>(graph: &'a G) -> impl 'a + Iterator<Item = G> {
    let v = graph.nodes().invert().smallest().unwrap();
    graph.nodes().iter().flat_map(move |u| {
        let mut g = graph.clone();
        g.add_node(v);
        let s = g.siblings(u);
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
    })
}

struct SplitDelSearcher<G> {
    found: Vec<G>
}

impl<G: Graph + Ord> SplitDelSearcher<G> {
    fn search_obstruction<'a>(&mut self, mut graph: G, mut edge_iter: sgtk::graph::EdgeIter<'a, G>) {
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
            //if candidate { // && is_obstruction(&graph) {
                ///self.found.push(graph);
                self.found.push(find_obstruction(graph));
            //}
        } else {
            self.found.push(find_obstruction(graph));
            /*if is_obstruction(&graph) {
                self.found.push(graph);
            }*/
        }
    }
}
