use crate::Graph16;
use crate::map::Map16;
use crate::bitset::Bitset16;
use crate::seq::Seq16;
use crate::planar;
use crate::embedding::*;

fn find_kuratowski(mut graph: Graph16) -> Graph16 {
    for (u, v) in graph.edges() {
        graph = graph.del_edge(u, v);
    }
}

struct TorusSearcher16 {
    embedding: RotationSystem16,
    admissible_faces: [Bitset16; 16],
    admissible_bridges: [Bitset16; 16],
    bridges: Map16<Graph16>,
}

impl TorusSearcher
