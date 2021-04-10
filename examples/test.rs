use std::collections::HashSet;
use sgtk::*;
use sgtk::graph::{minors, Graph, Graph16, Coloring, Coloring16};
use sgtk::embedding::RotationSystem;

fn main() {
    let mut k33 = Graph16::empty();
    for i in 0..6 {
        k33.add_node(i);
    }
    for i in 0..3 {
        for j in 3..6 {
            k33.add_edge(i, j);
        }
    }
    
    //dbg!(k33.nodes());
    //dbg!(&k33);
    let canon = k33.clone().to_canonical();
    dbg!(&canon);

    let graph = sgtk::parse::from_upper_tri("9 000001110000111000111111111111111000")
        .unwrap();
    dbg!(sgtk::toroidal::find_kuratowski(graph));
    dbg!(sgtk::toroidal::find_embedding(&graph));

    for minor in minors(&graph) {
        dbg!(minor);
        let k = sgtk::toroidal::find_kuratowski(minor);
        dbg!(&k);
        dbg!(sgtk::embedding::RotationSystem16::enumerate(&k).count());
        dbg!(sgtk::embedding::RotationSystem16::enumerate(&k)
            .filter(|embedding| embedding.genus() == 1)
            .count());
        dbg!(sgtk::toroidal::find_embedding(&minor));
    }

    viz::render_dot("test.pdf", &[k33, canon]);

    return;

    /*
    for n in 1..16 {
        let graph = Graph16::regular(n);
        /*
        let genus: usize = graph.components().map(|c| {
            planar::RotationSystem16::simple(&c).genus()
        }).sum();
        eprintln!("{}: genus {}", n, genus);
        */
        let embedding = embedding::RotationSystem16::simple(&graph);
        eprintln!("{}: genus {}", n, embedding.genus());
    }
    */

    dbg!(parse::from_graph6::<Graph16>("CF"));

    /*
    let mut graph = random::graph16(8); //Graph16::regular(8);
    while !graph.is_connected() || planar::fastdmp(&graph).is_some() {
        graph = random::graph16(10);
    }
    */
    /*
    let mut graph = Graph16::new(6);
    graph.add_edge(0, 1);
    graph.add_edge(0, 2);
    graph.add_edge(1, 3);
    graph.add_edge(1, 4);
    graph.add_edge(1, 5);
    graph.add_edge(2, 3);
    graph.add_edge(2, 4);
    graph.add_edge(2, 5);
    graph.add_edge(3, 4);
    graph.add_edge(3, 5);
    graph.add_edge(4, 5);
    */
    //let graph = parse::from_upper_tri("9 111000011100001100001000011111111111").unwrap();
        //Graph16::regular(8);
    let graph = parse::from_graph6::<Graph16>("H_?xqKw");

    dbg!(graph);

    //dbg!(toroidal::find_kuratowski(graph));

    //dbg!(embedding::RotationSystem16::enumerate(&graph).count());
    let mut graphs = Vec::new();
    graphs.push(graph);
    for minor in minors(&graph).filter(|minor| minor.is_connected()) {
        graphs.push(minor);
    }

    viz::render_dot("test.pdf", &graphs);
    let embedding = toroidal::find_embedding(&graph);

    dbg!(&embedding);

    if let Some(embedding) = embedding {
        dbg!(embedding.genus());
    }

    let mut graph2 = graph.clone();
    graph2.del_node(0);
    assert!(graph2.is_connected());
    dbg!(toroidal::find_embedding(&graph2));

    for minor in minors(&graph).filter(|minor| minor.is_connected()) {
        dbg!(parse::to_graph6(&minor));
        //dbg!(minor);
        let embedding = toroidal::find_embedding(&minor);
        //dbg!(&embedding);
        if embedding.is_none() {
            let k = toroidal::find_kuratowski(minor);
            let mut graphs = Vec::new();
            graphs.push(graph);
            graphs.push(minor);
            graphs.push(k);
            let mut minor_dif = minor.clone();
            minor.difference(&k);
            graphs.push(minor_dif);
            for bridge in toroidal::compute_bridges(&minor, &k) {
                graphs.push(bridge);
            }
            viz::render_dot("test.pdf", &graphs);
        }
        assert!(embedding.is_some());
    }

    /*
    let planar = graph.components().all(|c| {
        planar::RotationSystem16::enumerate(&c)
            .any(|embedding| embedding.genus() == 0)
    });

    eprintln!("Planar: {:?}", planar);

    eprintln!("Embedding count: {}", planar::RotationSystem16::enumerate(&graph).count());

    let embedding = planar::RotationSystem16::enumerate(&graph)
        .filter(|embedding| embedding.genus() == 0)
        .next().unwrap();

    dbg!(embedding);
    dbg!(embedding.count_faces());

    let embedding = planar::RotationSystem16::enumerate(&graph)
        .next().unwrap();

    dbg!(embedding);
    dbg!(embedding.count_faces());
    */
    /*
    for embedding in planar::RotationSystem16::enumerate(&graph) {
        eprintln!("genus {}", embedding.genus());
    }
    */

    /*
    for i in 0..9 {
        for j in i..9 {
            graph = graph.add_edge(i, j);
        }
    }
    */
    /*

    let mut graph = Graph16::new(9);

    graph = graph.add_edge(0, 1);
    graph = graph.add_edge(0, 3);

    graph = graph.add_edge(1, 2);
    graph = graph.add_edge(1, 3);
    graph = graph.add_edge(1, 4);
    graph = graph.add_edge(1, 5);

    graph = graph.add_edge(2, 5);

    graph = graph.add_edge(3, 4);
    graph = graph.add_edge(3, 6);
    graph = graph.add_edge(3, 7);

    graph = graph.add_edge(4, 5);
    graph = graph.add_edge(4, 7);

    graph = graph.add_edge(5, 7);
    graph = graph.add_edge(5, 8);

    graph = graph.add_edge(6, 7);

    graph = graph.add_edge(7, 8);


    let mut coloring = graph::Coloring16::new();
    coloring.set(0, 0);
    coloring.set(1, 0);
    coloring.set(2, 0);
    coloring.set(3, 0);
    coloring.set(4, 0);
    coloring.set(5, 0);
    coloring.set(6, 0);
    coloring.set(7, 0);
    coloring.set(8, 0);
    
    /*
    coloring.set(0, 1);
    coloring.set(1, 2);
    coloring.set(2, 3);
    coloring.set(3, 4);
    coloring.set(4, 5);
    coloring.set(5, 6);
    coloring.set(6, 7);
    coloring.set(7, 8);
    coloring.set(8, 9);
    coloring.set(8, 10);
    */

    //iso::search_tree(&graph, coloring, seq::Seq16::new());

    let mut graphs = Vec::new();
    /*
    graphs.push((graph, None)); 
    graphs.push((graph.to_canonical(), None)); 
    */

    let mut enumerator = enumeration::Enumerator16::new(9);
    enumerator.enumerate();
    eprintln!("Number of graphs: {}", enumerator.graphs.len());
    eprintln!("Number of calls to find_canonical: {}", enumerator.canonical_count);
    

    /*
    let mut enumerator = enumeration::Enumerator16::new(7);
    enumerator.enumerate();

    let graphs1 = enumerator.graphs;

    let mut enumerator = enumeration::Enumerator16::new(7);
    enumerator.auto_prune = false;
    enumerator.enumerate();

    let graphs2 = enumerator.graphs;

    for graph in graphs1 {
        if !graphs2.contains(&graph) {
            eprintln!("{:?}", graph);
            graphs.push((graph, None));
            graphs.push((graph.to_canonical(), None));

            let mut search_tree = iso::SearchTree::new(graph);
            graphs.push((search_tree.find_canonical(), None));
        }
    }
    */
    
    
    //let graph = Graph16 { g: [25, 98, 100, 25, 25, 38, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0] };

    //let graph = Graph16 { g: [67, 67, 52, 56, 28, 44, 67, 0, 0, 0, 0, 0, 0, 0, 0, 0] };
    let graph = Graph16 { g: [227, 143, 118, 186, 124, 61, 213, 203, 0, 0, 0, 0, 0, 0, 0, 0] };
    //let graph = Graph16 { g: [195, 135, 54, 184, 92, 108, 113, 139, 0, 0, 0, 0, 0, 0, 0, 0] };
    graphs.push((graph, None)); 
    graphs.push((graph.to_canonical(), None)); 

    //graphs.push((graph.to_canonical().to_canonical(), None)); 

    /*
    let graph1 = Graph16 { g: [201, 46, 310, 939, 244, 702, 721, 505, 396, 616, 0, 0, 0, 0, 0, 0] };
    let graph2 = Graph16 { g: [201, 46, 406, 939, 244, 762, 625, 445, 396, 616, 0, 0, 0, 0, 0, 0] };

    graphs.push((graph1, None)); 
    graphs.push((graph2, None)); 
    graphs.push((graph1.to_canonical(), None)); 
    eprintln!("next");
    graphs.push((graph2.to_canonical(), None)); 
    */

    //let graph = crate::random::graph16(10);
    //let graph = Graph16::new(16);
    /*
    let graph = Graph16::regular(16);
    graphs.push((graph, None)); 

    viz::render_dot("test.pdf", &graphs);

    graphs.push((graph.to_canonical(), None));
    */

    /*
    let perm = seq::Seq16::from_slice(&[2,8,6,4,5,3,7,0,1]);
    let graph2 = graph.shuffle2(&perm);

    graphs.push((graph, None)); //Some(coloring)));
    */

    /*
    coloring = iso::refinement_algorithm(&graph, coloring,
        seq::Seq16::from_slice(&[0]));
    graphs.push((graph, Some(coloring)));
    */

    /*
    graphs.push((graph2, None)); //Some(coloring)));

    graphs.push((iso::SearchTree::new(graph).find_canonical(), None));
    graphs.push((iso::SearchTree::new(graph2).find_canonical(), None));
    */

    
    /*
    let mut search = iso::SearchTree::new(graph);

    search.find_canonical();

    for graph in &search.leaf_graphs {
        graphs.push((*graph, None));
    }
    */

    /*
        
    let perm = seq::Seq16::from_slice(&[7,8,6,4,5,3,2,0,1]);
    let graph2 = graph.shuffle2(&perm);
    let canon2 = iso::SearchTree::new(graph2).find_canonical();

    graphs.push((graph2, Some(coloring)));
    graphs.push((canon2, Some(coloring)));
    */

    /*
    coloring = iso::refinement_algorithm(&graph, coloring,
        seq::Seq16::from_slice(&[1]));
    graphs.push((graph, Some(coloring)));

    coloring = iso::individualize(coloring, 1);
    graphs.push((graph, Some(coloring)));

    coloring = iso::refinement_algorithm(&graph, coloring,
        seq::Seq16::from_slice(&[3]));
    graphs.push((graph, Some(coloring)));
    */

    /*
    let newcol = iso::refinement(&graph, coloring,
        seq::Seq16::new());
    graphs.push((graph, Some(newcol)));

    let newcol = iso::refinement(&graph, coloring,
        seq::Seq16::from_slice(&[0]));
    graphs.push((graph, Some(newcol)));

    let newcol = iso::refinement(&graph, coloring,
        seq::Seq16::from_slice(&[0, 1]));
    graphs.push((graph, Some(newcol)));
    */

    /*
    for minor in graph.minors().take(3) {
        graphs.push((minor, Some(coloring)));
    }
    */

    viz::render_dot("test.pdf", &graphs);


    /*
    println!("minors:");

    for minor in graph.minors() {
        minor.print_dot();
        println!("");
    }
    */
    */
}
