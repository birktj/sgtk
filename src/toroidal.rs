use crate::Graph16;
use crate::map::Map16;
use crate::bitset::Bitset16;
use crate::seq::Seq16;
use crate::planar;
use crate::embedding::*;

pub fn find_kuratowski(mut graph: Graph16) -> Graph16 {
    for (u, v) in graph.edges() {
        graph.del_edge(u, v);
        if !graph.is_connected() {
            for component in graph.components() {
                if planar::fastdmp(&component).is_none() {
                    graph = component;
                    break;
                }
            }
        } else if planar::fastdmp(&graph).is_some() {
            graph.add_edge(u, v);
        }
    }
    graph
}

/*
struct TorusSearcher16 {
    embedding: RotationSystem16,
    admissible_faces: [Bitset16; 16],
    admissible_bridges: [Bitset16; 16],
    bridges: Map16<Graph16>,
    graph: Graph16,
}

impl TorusSearcher {
}
*/
