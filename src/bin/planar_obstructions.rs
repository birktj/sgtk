use sgtk::graph::{minors, Graph, Graph16};
use std::collections::HashSet;

fn find_planar_obstruction(graph: Graph16) -> Graph16 {
    for minor in minors(&graph) {
        if minor.is_connected() && sgtk::planar::fastdmp(&minor).is_none() {
            return find_planar_obstruction(minor)
        }
    }

    graph.to_canonical()
}

fn main() {
    let mut obstructions = HashSet::new();

    for _ in 0..100 {
        let graph: Graph16 = sgtk::random::graph(14);
        if !graph.is_connected() {
            continue
        }
        dbg!(&graph);
        if sgtk::planar::fastdmp(&graph).is_none() {
            obstructions.insert(find_planar_obstruction(graph));
        }
    }

    let obstructions: Vec<_> = obstructions.into_iter().collect();

    sgtk::viz::render_dot("test.pdf", &obstructions);

    /*
    let mut graph = sgtk::random::graph16(10);
    while !graph.is_connected() || sgtk::planar::fastdmp(&graph).is_some() {
        graph = sgtk::random::graph16(10);
    }

    let mut graphs = Vec::new();
    graphs.push((graph, None));
    sgtk::viz::render_dot("test.pdf", &graphs);
    graphs.push((sgtk::toroidal::find_kuratowski(graph).to_canonical(), None));
    sgtk::viz::render_dot("test.pdf", &graphs);
    */
}
