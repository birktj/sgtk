use crate::Graph16;
use crate::map::Map16;
use crate::smallvec::Smallvec;
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

#[inline(always)]
fn compute_bridges<'a>(graph: &'a Graph16, h: &'a Graph16) -> impl 'a + Iterator<Item = Graph16> {
    graph.edges_from_to(h.nodes(), h.nodes()).filter(move |(u, v)| {
        !h.has_edge(*u, *v)
    })
    .map(|(u, v)| {
        let mut g = Graph16::new(0);
        g.add_node(u);
        g.add_node(v);
        g.add_edge(u, v);
        g
    }).chain(graph.subgraph(h.nodes().invert()).components()
        .map(move |mut c| {
            c.union(&graph.neighbouring(c.nodes()).bipartite_split(c.nodes(), h.nodes()));
            c
        }))
}

pub fn find_embedding(graph: &Graph16) -> Option<RotationSystem16> {
    if let Some(embedding) = planar::fastdmp(graph) {
        return Some(embedding)
    }

    let h = find_kuratowski(*graph);

    for embedding in RotationSystem16::enumerate(&h).filter(|embedding| embedding.genus() == 1) {
        let mut searcher = TorusSearcher16::new(embedding, graph);
        if searcher.search() {
            return Some(searcher.embedding)
        }
    }
    None
}

struct TorusSearcher16<'a> {
    embedding: RotationSystem16,
    admissible_faces: [Bitset16; 16],
    admissible_bridges: [Bitset16; 16],
    bridges: Map16<Graph16>,
    faces: Map16<Face16>,
    graph: &'a Graph16,
    h: Graph16,
}

impl<'a> TorusSearcher16<'a> {
    fn new(embedding: RotationSystem16, graph: &'a Graph16) -> Self {
        let h = embedding.to_graph();
        let faces: Map16<_> = embedding.faces().collect();
        let bridges: Map16<_> = compute_bridges(graph, &h).collect();

        //dbg!(&bridges);

        let mut admissible_faces   = [Bitset16::new(); 16];
        let mut admissible_bridges = [Bitset16::new(); 16];

        for (i, bridge) in bridges.iter() {
            for (j, face) in faces.iter() {
                let attachments = h.nodes().intersection(&bridge.nodes());
                if embedding.face_nodes(*face).is_superset(&attachments) {
                    admissible_faces[i].set(j);
                    admissible_bridges[j].set(i);
                }
            }
        }
        Self {
            embedding,
            admissible_faces,
            admissible_bridges,
            bridges,
            faces,
            graph,
            h,
        }
    }

    fn update_faces(&mut self, bridge_i: usize, face_i: usize) -> bool {
        let attachments = self.h.nodes().intersection(&self.bridges[bridge_i].nodes());

        if self.embedding.face_nodes(self.faces[face_i]).is_superset(&attachments) {
            self.admissible_faces[bridge_i].set(face_i);
            self.admissible_bridges[face_i].set(bridge_i);
        }
        if self.admissible_faces[bridge_i].is_empty() {
            // No embedding
            false
        } else {
            true
        }
    }

    fn add_bridge(&mut self, bridge: Graph16, admissible_faces: Bitset16) -> Option<usize> {
        let i = self.bridges.push(bridge);
        let attachments = self.h.nodes().intersection(&bridge.nodes());

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

    fn remove_bridge(&mut self, i: usize) -> (Graph16, Bitset16) {
        let bridge = self.bridges.take(i).unwrap();
        let admissible_faces = self.admissible_faces[i];
        self.admissible_faces[i] = Bitset16::new();

        for j in self.admissible_bridges[i] {
            self.admissible_bridges[j].clear(i);
        }

        (bridge, admissible_faces)
    }

    fn remove_face(&mut self, i: usize) -> (Face16, Bitset16) {
        let face = self.faces.take(i).unwrap();
        let admissible_bridges = self.admissible_bridges[i];
        self.admissible_bridges[i] = Bitset16::new();

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

        let old_admissible_faces = self.admissible_faces[bridge_i];
        self.admissible_faces[bridge_i] = Bitset16::new();

        for j in old_admissible_faces {
            self.admissible_bridges[j].clear(bridge_i);
        }

        for face_i in old_admissible_faces {
            //dbg!(old_admissible_faces);
            //dbg!(face_i);
            //dbg!(bridge);
            //dbg!(self.embedding.face(face).collect::<Vec<_>>());

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
                    let u = usize::from(*u);
                    self.embedding.embed_edge_after(start, u, end, start);
                    self.h.add_node(end);
                    self.h.add_edge(start, end);

                    //dbg!(&self.embedding);

                    let mut new_bridges_idx = Bitset16::new();

                    let mut new_faces_idx = Bitset16::new();
                    new_faces_idx.set(face_i);
    
                    let mut ok = true;

                    for new_bridge in compute_bridges(&bridge, &self.h.clone()) {
                        if let Some(i) = self.add_bridge(new_bridge, new_faces_idx) {
                            new_bridges_idx.set(i);
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
                let path = bridge.path(start, attachments).unwrap();
                //dbg!(path);
                let end = path.last().unwrap();
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
                        let u = usize::from(*u);
                        let v = usize::from(*v);
                        //dbg!(u, v);
                        let new_faces = self.embedding.embed_bisecting_path_after(face, &path, u, v);
                        let oldh = self.h.clone();
                        self.h.union(&Graph16::from_path(&path));

                        let mut new_faces_idx = Bitset16::new();
                        new_faces_idx.set(self.faces.push(new_faces[0]));
                        new_faces_idx.set(self.faces.push(new_faces[1]));

                        let mut ok = true;

                        for bridge_j in old_admissible_bridges {
                            for face_j in new_faces_idx {
                                if !self.update_faces(bridge_j, face_j) {
                                    ok = false;
                                    break;
                                }
                            }
                            if !ok {
                                break
                            }
                        }

                        let mut new_bridges_idx = Bitset16::new();

                        if ok {
                            for new_bridge in compute_bridges(&bridge, &self.h.clone()) {
                                if let Some(i) = self.add_bridge(new_bridge, new_faces_idx) {
                                    new_bridges_idx.set(i);
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
                        for (u, v) in Graph16::from_path(&path).edges() {
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
                self.admissible_bridges[face_i] = old_admissible_bridges;
                for bridge_j in old_admissible_bridges {
                    self.admissible_faces[bridge_j].set(face_i);
                }
            }
        }
        self.bridges.insert(bridge_i, bridge);
        self.admissible_faces[bridge_i] = old_admissible_faces;

        for j in old_admissible_faces {
            self.admissible_faces[j].set(bridge_i);
        }
        false
    }
}
