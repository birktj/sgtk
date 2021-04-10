use crate::graph::Graph;
use crate::map::Map64;
use crate::seq::{Seq, Seq16};
use crate::planar;
use crate::embedding::*;
use crate::bitset::{Bitset, Bitset64};

pub fn find_kuratowski<G: Graph>(mut graph: G) -> G {
    for (u, v) in graph.clone().edges() {
        graph.del_edge(u, v);
        if !graph.is_connected() {
            for component in graph.clone().components() {
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

#[inline(always)]
pub fn compute_bridges<'a, G: Graph>(graph: &'a G, h: &'a G) -> impl 'a + Iterator<Item = G> {
    graph.edges_from_to(h.nodes(), h.nodes()).filter(move |(u, v)| {
        !h.has_edge(*u, *v)
    })
    .map(|(u, v)| {
        let mut g = G::empty();
        g.add_node(u);
        g.add_node(v);
        g.add_edge(u, v);
        g
    }).chain(graph.subgraph(&h.nodes().invert()).components()
        .map(move |mut c| {
            c.union(&graph.neighbouring(&c.nodes()).bipartite_split(&c.nodes(), &h.nodes()));
            c
        }))
}

pub fn find_embedding<G: Graph>(graph: &G) -> Option<G::Embedding> {
    let node_count = graph.nodes().count();
    let edge_count = graph.edges().count();

    if edge_count > 3*node_count {
        return None
    }

    if let Some(embedding) = planar::fastdmp(graph) {
        return Some(embedding)
    }

    let h = find_kuratowski(graph.to_owned());

    for mut bridge in graph.subgraph(&h.nodes().invert()).components() {
        let attachments = graph.neighbouring(&h.nodes())
            .nodes().intersection(&bridge.nodes());
        let u = bridge.nodes().invert().smallest().unwrap();
        bridge.add_node(u);
        bridge.add_edges(u, &attachments);
        if planar::fastdmp(&bridge).is_none() {
            return None
        }
    }

    for embedding in G::Embedding::enumerate(&h).filter(|embedding| embedding.genus() == 1) {
        if let Some(mut searcher) = TorusSearcher::new(embedding, graph) {
            if searcher.search() {
                return Some(searcher.embedding)
            }
        }
    }
    None
}

struct TorusSearcher<G: Graph> {
    embedding: G::Embedding,
    admissible_faces: Map64<Bitset64>, //[Bitset16; 16],
    admissible_bridges: Map64<Bitset64>, // [HashSet<usize>; 16],
    bridges: Map64<G>,
    faces: Map64<Face>,
    h: G,
}

impl<G: Graph> TorusSearcher<G> {
    fn new(embedding: G::Embedding, graph: &G) -> Option<Self> {
        let h = embedding.to_graph();
        let faces: Map64<_> = embedding.faces().collect();
        let bridges: Map64<_> = compute_bridges(graph, &h).collect();

        //dbg!(&bridges);

        let mut admissible_faces   = Map64::new(); //::<Bitset16>::new(); //  [Bitset16::new(); 16];
        let mut admissible_bridges = Map64::new(); // [HashSet<usize>; 16] = Default::default();

        for (i, _) in faces.iter() {
            admissible_bridges.insert(i, Bitset64::new());
        }
        for (i, bridge) in bridges.iter() {
            admissible_faces.insert(i, Bitset64::new());
            for (j, face) in faces.iter() {
                let attachments = h.nodes().intersection(&bridge.nodes());
                if embedding.face_nodes(*face).is_superset(&attachments) {
                    admissible_faces[i].set(j);
                    admissible_bridges[j].set(i);
                }
            }
            if admissible_faces[i].is_empty() {
                return None
            }
        }
        Some(Self {
            embedding,
            admissible_faces,
            admissible_bridges,
            bridges,
            faces,
            h,
        })
    }

    fn update_faces(&mut self, bridge_i: usize, face_i: usize) {
        let attachments = self.h.nodes().intersection(&self.bridges[bridge_i].nodes());

        if self.embedding.face_nodes(self.faces[face_i]).is_superset(&attachments) {
            self.admissible_faces[bridge_i].set(face_i);
            self.admissible_bridges[face_i].set(bridge_i);
        }
    }

    fn add_bridge(&mut self, bridge: G, admissible_faces: Bitset64) -> Option<usize> {
        let attachments = self.h.nodes().intersection(&bridge.nodes());
        let i = self.bridges.push(bridge);

        self.admissible_faces.insert(i, Bitset64::new());

        for j in admissible_faces {
            if self.embedding.face_nodes(self.faces[j]).is_superset(&attachments) {
                self.admissible_faces[i].set(j);
                self.admissible_bridges[j].set(i);
            }
        }
        if self.admissible_faces[i].is_empty() {
            // No embedding
            self.remove_bridge(i);
            None
        } else {
            Some(i)
        }
    }

    fn remove_bridge(&mut self, i: usize) -> (G, Bitset64) {
        let bridge = self.bridges.take(i).unwrap();
        let admissible_faces = self.admissible_faces.take(i).unwrap();
        //self.admissible_faces[i] = Bitset16::new();

        for j in admissible_faces {
            self.admissible_bridges[j].clear(i);
        }

        (bridge, admissible_faces)
    }

    fn remove_face(&mut self, i: usize) -> (Face, Bitset64) {
        let face = self.faces.take(i).unwrap();
        let admissible_bridges = self.admissible_bridges.take(i).unwrap();
        //self.admissible_bridges[i] = Bitset16::new();

        for j in admissible_bridges {
            self.admissible_faces[j].clear(i);
        }

        (face, admissible_bridges)
    }

    fn search(&mut self) -> bool {
        //dbg!("search");
        //dbg!(&self.embedding);
        //dbg!(&self.faces);
        //dbg!(&self.bridges);
        if self.bridges.is_empty() {
            return true
        }
        let (bridge_i, bridge) = self.bridges.pop().unwrap();
        let bridge_nodes = bridge.nodes();

        //let old_admissible_faces = self.admissible_faces[bridge_i];
        //self.admissible_faces[bridge_i] = Bitset16::new();
        let old_admissible_faces = self.admissible_faces.take(bridge_i).unwrap();

        for j in old_admissible_faces {
            self.admissible_bridges[j].clear(bridge_i);
        }

        for face_i in old_admissible_faces {
            //dbg!(old_admissible_faces);
            //dbg!(face_i);
            //dbg!(bridge);
            //dbg!(self.embedding.face(self.faces[*face_i]).collect::<Vec<_>>());

            let mut attachments = bridge_nodes.intersection(&self.h.nodes());
            let start = attachments.smallest().unwrap();
            attachments.clear(start);

            if attachments.is_empty() {
                let face = self.faces[face_i];
                let end = bridge.siblings(start).smallest().unwrap();
                let mut start_endpoints = Seq16::new();
                for (u, v) in self.embedding.face(face) {
                    if u == start {
                        start_endpoints.push(v);
                    }
                }
                for u in start_endpoints.iter() {
                    self.embedding.embed_edge_before(start, u, end);
                    self.h.add_node(end);
                    self.h.add_edge(start, end);

                    let mut new_bridges_idx = Vec::new();

                    let mut new_faces_idx = Bitset64::new();
                    new_faces_idx.set(face_i);
    
                    let mut ok = true;

                    for new_bridge in compute_bridges(&bridge, &self.h.clone()) {
                        if let Some(i) = self.add_bridge(new_bridge, new_faces_idx) {
                            new_bridges_idx.push(i);
                        } else {
                            ok = false;
                            break;
                        }
                    }

                    if ok && self.search() {
                        return true
                    }
                    self.h.del_edge(start, end);
                    self.h.del_node(end);
                    self.embedding.remove_edge(start, end);

                    for j in new_bridges_idx {
                        self.remove_bridge(j);
                    }
                }
            } else {
                let (face, old_admissible_bridges) = self.remove_face(face_i);
                let path = bridge.path(start, &attachments).unwrap();
                //dbg!(path);
                let end = path.get(path.len()-1);
                let mut start_endpoints = Seq16::new();
                let mut end_endpoints = Seq16::new();
                for (u, v) in self.embedding.face(face) {
                    if u == start {
                        start_endpoints.push(v);
                    }
                    if v == end {
                        end_endpoints.push(u);
                    }
                }
                for u in start_endpoints.iter() {
                    for v in end_endpoints.iter() {
                        //dbg!(u, v);
                        let oldh = self.h.clone();

                        let new_faces = self.embedding.embed_bisecting_path_after(&path, u, v);
                        self.h.union(&G::from_path(&path));

                        let mut new_faces_idx = Bitset64::new();
                        new_faces_idx.set(self.faces.push(new_faces[0]));
                        new_faces_idx.set(self.faces.push(new_faces[1]));

                        for idx in new_faces_idx {
                            self.admissible_bridges.insert(idx, Bitset64::new());
                        }

                        let mut ok = true;

                        for bridge_j in old_admissible_bridges {
                            for face_j in new_faces_idx {
                                self.update_faces(bridge_j, face_j);
                            }
                            if self.admissible_faces[bridge_j].is_empty() {
                                ok = false;
                                break
                            }
                        }

                        let mut new_bridges_idx = Vec::new();

                        if ok {
                            for new_bridge in compute_bridges(&bridge, &self.h.clone()) {
                                if let Some(i) = self.add_bridge(new_bridge, new_faces_idx) {
                                    new_bridges_idx.push(i);
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                        }

                        if ok && self.search() {
                            return true
                        }

                        self.h = oldh;
                        //dbg!(&path);
                        for (u, v) in G::from_path(&path).edges() {
                            self.embedding.remove_edge(u, v);
                        }

                        for j in new_faces_idx {
                            //dbg!("remove", j);
                            self.remove_face(j);
                        }
                        for j in new_bridges_idx {
                            self.remove_bridge(j);
                        }
                    }
                }
                //dbg!("insert", face_i);
                self.faces.insert(face_i, face);
                self.admissible_bridges.insert(face_i, old_admissible_bridges);
                for bridge_j in self.admissible_bridges[face_i] {
                    self.admissible_faces[bridge_j].set(face_i);
                }
            }
        }
        self.bridges.insert(bridge_i, bridge);
        self.admissible_faces.insert(bridge_i, old_admissible_faces);

        for j in self.admissible_faces[bridge_i] {
            self.admissible_bridges[j].set(bridge_i);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{minors, Graph16};

    #[test]
    fn check_known_minor() {
        let graph: Graph16 = crate::parse::from_upper_tri("9 000001110000111000111111111111111000")
            .unwrap();
        assert!(find_embedding(&graph).is_none());
        for minor in minors(&graph) {
            assert!(find_embedding(&minor).is_some());
        }
    }

}
