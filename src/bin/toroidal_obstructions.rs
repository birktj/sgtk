use sgtk::Graph16;
use std::collections::HashSet;

fn find_toroidal_obstruction(graph: Graph16) -> Graph16 {
    for minor in graph.minors().filter(|minor| minor.is_connected()) {
        if sgtk::toroidal::find_embedding(&minor).is_none() {
            return find_toroidal_obstruction(minor)
        }
    }

    graph.to_canonical()
}

fn main() {
    let mut known_obstructions = HashSet::new();
    for line in std::fs::read_to_string("torus-obstructions.txt").unwrap().lines() {
        if let Some(obstruction) = sgtk::parse::from_upper_tri(line) {
            /*
            dbg!(line);
            if !obstruction.is_connected() {
                continue
            }
            //dbg!(&obstruction);
            assert!(sgtk::toroidal::find_embedding(&obstruction).is_none());
            for minor in obstruction.subgraphs().filter(|minor| minor.is_connected()) {
                //dbg!(&minor);
                assert!(sgtk::toroidal::find_embedding(&minor).is_some());
            }
            */
            known_obstructions.insert(obstruction.to_canonical());
        }
    }

    let mut obstructions = HashSet::new();

    for _ in 0..100 {
        let graph = sgtk::random::graph16(15);
        if !graph.is_connected() {
            continue
        }
        //dbg!(&graph);
        if sgtk::toroidal::find_embedding(&graph).is_none() {
            let obstruction = find_toroidal_obstruction(graph);
            assert!(sgtk::toroidal::find_embedding(&obstruction).is_none());
            if known_obstructions.contains(&obstruction) {
                eprintln!("It is known");
            } else {
                dbg!(sgtk::parse::to_graph6(&obstruction));
                eprintln!("It is not known");
                obstructions.insert(obstruction);
            }
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
