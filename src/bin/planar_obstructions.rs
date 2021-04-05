use sgtk::Graph16;
use std::collections::HashSet;

fn find_planar_obstruction(graph: Graph16) -> Graph16 {
    for minor in graph.minors() {
        if minor.is_connected() && sgtk::planar::fastdmp(&minor).is_none() {
            return find_planar_obstruction(minor)
        }
    }

    graph.to_canonical()
}

fn main() {
    /*
    let mut obstructions = HashSet::new();

    for _ in 0..10 {
        let graph = sgtk::random::graph16(10);
        if !graph.is_connected() {
            continue
        }
        if sgtk::planar::fastdmp(&graph).is_none() {
            obstructions.insert(find_planar_obstruction(graph));
        }
    }

    let obstructions = obstructions.into_iter()
        .map(|g| (g, None)).collect::<Vec<_>>();

    sgtk::viz::render_dot("test.pdf", &obstructions);
    */

    let mut graph = sgtk::random::graph16(10);
    while !graph.is_connected() || sgtk::planar::fastdmp(&graph).is_some() {
        graph = sgtk::random::graph16(10);
    }

    let mut graphs = Vec::new();
    graphs.push((graph, None));
    sgtk::viz::render_dot("test.pdf", &graphs);
    graphs.push((sgtk::toroidal::find_kuratowski(graph).to_canonical(), None));
    sgtk::viz::render_dot("test.pdf", &graphs);
}
