use sgtk::*;

fn main() {
    let mut enumerator = enumeration::Enumerator16::new(9)
        .set_prune(|g| {
            if g.is_connected() {
                //toroidal::find_embedding(g).is_none()
                planar::fastdmp(g).is_none()
            } else {
                false
            }
        });
        //.set_prune(|g| !g.is_connected()); // || planar::fastdmp(g).is_none());

    enumerator.enumerate();

    enumerator.graphs.retain(|g| g.is_connected());

    dbg!(enumerator.counts);
    println!("{}", enumerator.graphs.len());

    /*
    let graphs = enumerator.graphs.into_iter()
        .map(|graph| (graph, None))
        .collect::<Vec<_>>();

    viz::render_dot("test.pdf", &graphs);
    */
}

/*
use std::collections::HashSet;
use sgtk::Graph16;
use sgtk::planar;
use sgtk::bitset::{Bitset, Bitset16};

pub struct Enumerator16 {
    maxn: usize,
    //pub graphs: Vec<Graph16>,
    pub graphs: HashSet<Graph16>,
    pub auto_prune: bool,
    pub canonical_count: u32,
}

impl Enumerator16 {
    pub fn new(maxn: usize) -> Enumerator16 {
        Enumerator16 {
            maxn,
            //graphs: Vec::new(),
            graphs: HashSet::new(),
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
            let mut graph = graph;
            graph.add_edges(n, edges); //.to_canonical();

            if !graph.is_connected() {
                continue
            }

            let graph = graph.to_canonical();
            self.canonical_count += 1;

            if self.graphs.contains(&graph) {
                continue
            }

            //assert_eq!(graph, graph.to_canonical(), "original: {:?}", orig_graph);
            
            //if !self.graphs.contains(&graph) && planar::dmp(&graph).is_some() {
            if planar::fastdmp(&graph).is_some() {
                self.graphs.insert(graph);
                self.enumerate_inner(graph, n+1);
            }
        }
    }

    pub fn enumerate(&mut self) {
        let graph = Graph16::new(1);
        self.graphs.insert(graph);
        self.enumerate_inner(graph, 1);
    }
}

fn main() {
    /*
    let orig_graphs = std::fs::read_to_string("planar_conn.7.g6")
        .unwrap()
        .lines()
        .map(|l| sgtk::parse::from_graph6(l).to_canonical())
        .collect::<HashSet<_>>();
    */

    let mut enumerator = Enumerator16::new(9);
    enumerator.enumerate();
    dbg!(&enumerator.graphs.len());

    let mut graphs = Vec::new();
    for graph in &enumerator.graphs {
        if graph.nodes().count() == 9 { // && sgtk::planar2::fastdmp(graph).is_some() { 
            //&& !orig_graphs.contains(graph) {
        //if planar::dmp(&graph).is_none() {
            //graphs.push((graph, None));
            //graphs.push((*graph, None));
            graphs.push(*graph);
        //}
        }
    }
    dbg!(graphs.len());
    /*
    for graph in &orig_graphs {
        if !enumerator.graphs.contains(graph) {
            dbg!(planar2::fastdmp(graph));
            dbg!(sgtk::parse::to_graph6(graph));
            //dbg!(planar::dmp(graph));
            //dbg!(planar::dmp(graph).unwrap().genus());
            graphs.push((*graph, None));
        }

    }
    dbg!(graphs.len());
    */

    //let mut graphs = Vec::new();
    //graphs.push((sgtk::parse::from_graph6("F@|ZO").to_canonical(), None));
    //graphs.push((sgtk::parse::from_graph6("FJn^W").to_canonical(), None));

    //dbg!(planar2::fastdmp(&graphs[0].0));

    //viz::render_dot("test.pdf", &graphs);
}
*/
