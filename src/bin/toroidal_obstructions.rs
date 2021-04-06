use sgtk::Graph16;
use std::collections::HashSet;

fn find_toroidal_obstruction(graph: Graph16) -> Graph16 {
    for minor in graph.minors() {
        if minor.is_connected() && sgtk::toroidal::find_embedding(&minor).is_none() {
            return find_toroidal_obstruction(minor)
        }
    }

    graph.to_canonical()
}

fn main() {
    let mut obstructions = HashSet::new();

    for _ in 0..10000 {
        let graph = sgtk::random::graph16(8);
        if !graph.is_connected() {
            continue
        }
        //dbg!(&graph);
        if sgtk::toroidal::find_embedding(&graph).is_none() {
            obstructions.insert(find_toroidal_obstruction(graph));
        }
    }

    let obstructions = obstructions.into_iter()
        .map(|g| (g, None)).collect::<Vec<_>>();

    dbg!(obstructions.len());

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
