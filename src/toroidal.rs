use crate::map::{Map64, DynMap, FullMapError};
use crate::seq::Seq16;
use crate::planar;
use crate::bitset::{Bitset64, DynIntSet};
use crate::embedding::Face;
use crate::prelude::*;

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
pub fn compute_bridges<'a, G: Graph>(graph: &'a G, h: &'a G, h_nodes: &'a G::Set) -> impl 'a + Iterator<Item = G> {
    graph.edges_from_to(h_nodes.clone(), h_nodes.clone()).filter(move |(u, v)| {
        !h.has_edge(*u, *v)
    })
    .map(|(u, v)| {
        let mut g = G::empty();
        g.add_node(u);
        g.add_node(v);
        g.add_edge(u, v);
        g
    }).chain(graph.subgraph(&h_nodes.invert()).components()
        .map(move |mut c| {
            c.union(&graph.neighbouring(&c.nodes()).bipartite_split(&c.nodes(), &h_nodes));
            c
        }))
}

use crate::smallvec::Smallvec;
pub fn graph16_bridges(graph: &crate::graph::Graph16, h: &crate::graph::Graph16) -> Smallvec<crate::graph::Graph16, 16> {
    let mut res = Smallvec::new();
    //for bridge in compute_bridges(graph, h, &h.nodes()) {
    //for mut c in graph.subgraph(&h.nodes().invert()).components() {
    //for mut c in graph.components() {
    let h_nodes = h.nodes();
    for (u, v) in graph.edges_from_to(h_nodes, h_nodes) {
        let mut bridge = crate::graph::Graph16::empty();
        bridge.add_node(u);
        bridge.add_node(v);
        bridge.add_edge(u, v);

        //c.union(&graph.neighbouring(&c.nodes()).bipartite_split(&c.nodes(), &h.nodes()));
        res.push(bridge);
    }
    res
}

pub struct Embedder<G: Graph> {
    subgraph: Option<G>,
    subgraph_embeddings: Vec<G::Embedding>,
    pub filter_all: bool,
}

pub struct EmbeddingResult<G: Graph> {
    pub embedding: Option<G::Embedding>,
    pub filtered_embeddings: Vec<bool>,
}

impl<G: Graph> EmbeddingResult<G> {
    fn empty() -> Self {
        Self {
            embedding: None,
            filtered_embeddings: Vec::new(),
        }
    }
}

impl<G: Graph> Embedder<G> {
    pub fn new() -> Self {
        Self {
            subgraph: None,
            subgraph_embeddings: Vec::new(),
            filter_all: false,
        }
    }

    pub fn add_subgraph(&mut self, subgraph: G) {
        self.subgraph = Some(subgraph);
    }

    pub fn add_subgraph_embeddings(&mut self, embeddings: &[G::Embedding]) {
        self.subgraph_embeddings = embeddings.to_owned();
    }

    pub fn find_embedding(self, graph: &G) -> EmbeddingResult<G> {
        let node_count = graph.nodes().count();
        let edge_count = graph.edges().count();

        if edge_count > 3*node_count {
            return EmbeddingResult::empty()
        }

        if graph.is_connected() {
            if let Some(embedding) = planar::fastdmp(graph) {
                return EmbeddingResult {
                    embedding: Some(embedding),
                    filtered_embeddings: Vec::new(),
                }
            }
            self.find_embedding_connected(graph)
        } else {
            let mut embedding = G::Embedding::empty();
            let mut non_planar_component = None;
            for component in graph.to_owned().components() {
                if let Some(planar) = planar::fastdmp(&component) {
                    embedding.embed_disconnected(&planar);
                } else if non_planar_component.is_none() {
                    non_planar_component = Some(component)
                } else {
                    return EmbeddingResult::empty()
                }
            }
            if let Some(component) = non_planar_component {
                let mut res = self.find_embedding_connected(&component);
                res.embedding.as_mut().map(|e| e.embed_disconnected(&embedding));
                res
            } else {
                EmbeddingResult {
                    embedding: Some(embedding),
                    filtered_embeddings: Vec::new(),
                }
            }
        }
    }

    fn find_embedding_connected(self, graph: &G) -> EmbeddingResult<G> {
        let node_count = graph.nodes().count();
        let edge_count = graph.edges().count();

        if edge_count > 3*node_count {
            return EmbeddingResult::empty()
        }

        let h = self.subgraph.clone()
            .filter(|h| graph.is_supergraph(&h))
            .unwrap_or_else(|| {
                find_kuratowski(graph.clone())
            });

        if self.subgraph_embeddings.is_empty() {
            let node_count = h.nodes().count();
            let edge_count = h.edges().count();

            self.find_embedding_with_embeddings(graph, &h, G::Embedding::enumerate(&h)
                .filter(|embedding| (3 + edge_count - node_count - embedding.faces().count())/2 == 1))
        } else {
            let subgraph_embeddings = self.subgraph_embeddings.clone().into_iter();
            self.find_embedding_with_embeddings(graph, &h, subgraph_embeddings)
        }
    }

    fn find_embedding_with_embeddings<I: Iterator<Item = G::Embedding>>(self, graph: &G, h: &G, subgraph_embeddings: I) -> EmbeddingResult<G> {
        let mut filtered_embeddings = vec![false; self.subgraph_embeddings.len()];
        let mut i = 0;

        let h_nodes = h.nodes();

        let mut bridges = Vec::new();

        for bridge in compute_bridges(graph, h, &h_nodes) {
            let attachments = h_nodes.intersection(&bridge.nodes());
            let mut test_bridge = bridge.clone();
            test_bridge.merge_nodes(&attachments);
            if crate::planar::fastdmp(&test_bridge).is_none() {
                return EmbeddingResult::empty()
            }
            bridges.push(bridge);
            /*
            if attachments.count() <= 2 && bridge.nodes().count() > attachments.count() {
                let mut new_bridge = G::empty();
                for u in attachments.iter() {
                    new_bridge.add_node(u);
                }
                let u = bridge.nodes().intersection(&attachments.invert()).smallest().unwrap();
                new_bridge.add_node(u);
                new_bridge.add_edges(u, &attachments);
                bridges.push(bridge);
            } else {
                bridges.push(bridge);
            }
            */
        }

        let mut res_embedding = None;

        for embedding in subgraph_embeddings {
            if let Ok(res) = search_embedding::<G, Bitset64, Map64<Bitset64>, Map64<G>, Map64<Face>>(embedding.clone(), &bridges) {
                if let Some(embedding) = res {
                    res_embedding = Some(embedding);
                    if !self.filter_all {
                        break
                    }
                } else if i < filtered_embeddings.len() {
                    filtered_embeddings[i] = true;
                }
            } else if let Some(embedding) = search_embedding::<G, DynIntSet, DynMap<DynIntSet>, DynMap<G>, DynMap<Face>>(embedding, &bridges).unwrap() {
                res_embedding = Some(embedding);
                if !self.filter_all {
                    break
                }
            } else if i < filtered_embeddings.len() {
                filtered_embeddings[i] = true;
            }

            i += 1;
        }

        EmbeddingResult {
            embedding: res_embedding,
            filtered_embeddings
        }
    }
}

pub fn find_embedding<G: Graph>(graph: &G) -> Option<G::Embedding> {
    Embedder::new().find_embedding(graph).embedding
}

fn search_embedding<G: Graph, S: Intset, SM, BM, FM>(embedding: G::Embedding, bridges: &[G]) -> Result<Option<G::Embedding>, FullMapError>
    where SM: Slotmap<Output = S>,
          BM: Slotmap<Output = G>,
          FM: Slotmap<Output = Face>,
          for<'a> &'a SM: IntoIterator<Item = (usize, &'a SM::Output)>,
          for<'a> &'a BM: IntoIterator<Item = (usize, &'a G)>,
          for<'a> &'a FM: IntoIterator<Item = (usize, &'a Face)>,
          for<'a> &'a SM::Output: IntoIterator<Item = usize>,
{
    if let Some(mut searcher) = TorusSearcher::<G, S, SM, BM, FM>::new(embedding, bridges)? {
        if searcher.search()? {
            Ok(Some(searcher.embedding))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

struct TorusSearcher<G: Graph, S, SM, BM, FM> {
    embedding: G::Embedding,
    admissible_faces: SM, //[Bitset16; 16],
    admissible_bridges: SM, // [HashSet<usize>; 16],
    one_admissible: S,
    bridges: BM,
    faces: FM,
    h: G,
    h_nodes: G::Set,
    bridges_rem: usize,
}

impl<G: Graph, S: Intset, SM, BM, FM> TorusSearcher<G, S, SM, BM, FM>
    where SM: Slotmap<Output = S>,
          BM: Slotmap<Output = G>,
          FM: Slotmap<Output = Face>,
          for<'a> &'a SM: IntoIterator<Item = (usize, &'a SM::Output)>,
          for<'a> &'a BM: IntoIterator<Item = (usize, &'a G)>,
          for<'a> &'a FM: IntoIterator<Item = (usize, &'a Face)>,
          for<'a> &'a SM::Output: IntoIterator<Item = usize>,
{
    fn new(embedding: G::Embedding, bridges_list: &[G]) -> Result<Option<Self>, FullMapError> {
        let h = embedding.to_graph();
        let h_nodes = h.nodes();
        let mut faces = FM::new();
        for face in embedding.faces() {
            faces.push(face)?;
        }
        let mut bridges = BM::new();
        for bridge in bridges_list {
            bridges.push(bridge.to_owned())?;
        }

        //dbg!(&bridges);

        let mut admissible_faces   = SM::new(); //::<Bitset16>::new(); //  [Bitset16::new(); 16];
        let mut admissible_bridges = SM::new(); // [HashSet<usize>; 16] = Default::default();
        let mut one_admissible     = SM::Output::new();

        for (i, _) in &faces {
            admissible_bridges.insert(i, SM::Output::new());
        }
        for (i, bridge) in &bridges {
            admissible_faces.insert(i, SM::Output::new());
            for (j, face) in &faces {
                let attachments = h_nodes.intersection(&bridge.nodes());
                if embedding.face_nodes(*face).is_superset(&attachments) {
                    admissible_faces[i].set(j);
                    admissible_bridges[j].set(i);
                }
            }
            if admissible_faces[i].is_empty() {
                return Ok(None)
            } else if admissible_faces[i].count() == 1 {
                one_admissible.set(i);
            }
        }
        Ok(Some(Self {
            embedding,
            admissible_faces,
            admissible_bridges,
            one_admissible,
            bridges,
            faces,
            h,
            h_nodes,
            bridges_rem: bridges_list.len(),
        }))
    }

    fn update_faces(&mut self, bridge_i: usize, face_i: usize) {
        let attachments = self.h_nodes.intersection(&self.bridges[bridge_i].nodes());

        if self.embedding.face_nodes(self.faces[face_i]).is_superset(&attachments) {
            self.admissible_faces[bridge_i].set(face_i);
            self.admissible_bridges[face_i].set(bridge_i);
        }
    }

    fn add_bridge(&mut self, bridge: G, admissible_faces: &SM::Output) -> Result<Option<usize>, FullMapError> {
        let attachments = self.h_nodes.intersection(&bridge.nodes());
        let i = self.bridges.push(bridge)?;

        self.admissible_faces.insert(i, SM::Output::new());

        for j in admissible_faces {
            if self.embedding.face_nodes(self.faces[j]).is_superset(&attachments) {
                self.admissible_faces[i].set(j);
                self.admissible_bridges[j].set(i);
            }
        }
        if self.admissible_faces[i].is_empty() {
            // No embedding
            self.remove_bridge(i);
            Ok(None)
        } else {
            if self.admissible_faces[i].count() == 1 {
                self.one_admissible.set(i);
            }
            Ok(Some(i))
        }
    }

    fn remove_bridge(&mut self, i: usize) -> (G, SM::Output) {
        let bridge = self.bridges.take(i).unwrap();
        let admissible_faces = self.admissible_faces.take(i).unwrap();
        //self.admissible_faces[i] = Bitset16::new();
        self.one_admissible.clear(i);

        for j in &admissible_faces {
            self.admissible_bridges[j].clear(i);
        }

        (bridge, admissible_faces)
    }

    fn remove_face(&mut self, i: usize) -> (Face, SM::Output) {
        let face = self.faces.take(i).unwrap();
        let admissible_bridges = self.admissible_bridges.take(i).unwrap();
        //self.admissible_bridges[i] = Bitset16::new();

        for j in &admissible_bridges {
            self.admissible_faces[j].clear(i);
        }

        (face, admissible_bridges)
    }

    fn search(&mut self) -> Result<bool, FullMapError> {
        //dbg!("search");
        //dbg!(&self.embedding);
        //dbg!(&self.faces);
        //dbg!(&self.bridges);
        if self.bridges.is_empty() {
            return Ok(true)
        }
        let (bridge_i, bridge) = if let Some(i) = self.one_admissible.smallest() {
            self.one_admissible.clear(i);
            (i, self.bridges.take(i).unwrap())
        } else {
            self.bridges.pop().unwrap()
        };
        let bridge_nodes = bridge.nodes();

        //let old_admissible_faces = self.admissible_faces[bridge_i];
        //self.admissible_faces[bridge_i] = Bitset16::new();
        let old_admissible_faces = self.admissible_faces.take(bridge_i).unwrap();

        for j in &old_admissible_faces {
            self.admissible_bridges[j].clear(bridge_i);
        }
        let mut attachments = bridge_nodes.intersection(&self.h_nodes);
        let start = attachments.smallest().unwrap();
        attachments.clear(start);

        if attachments.is_empty() {
            let face_i = old_admissible_faces.smallest().unwrap();
            let face = self.faces[face_i];
            let end = bridge.siblings(start).smallest().unwrap();
            let mut start_endpoint = 0;
            for (u, v) in self.embedding.face(face) {
                if u == start {
                    start_endpoint = v;
                    break;
                }
            }
            self.embedding.embed_edge_before(start, start_endpoint, end);
            self.h_nodes.set(end);
            self.h.add_node(end);
            self.h.add_edge(start, end);

            let mut new_bridges_idx = SM::Output::new();

            let mut new_faces_idx = SM::Output::new();
            new_faces_idx.set(face_i);

            let mut ok = true;

            for new_bridge in compute_bridges(&bridge, &self.h.clone(), &self.h_nodes.clone()) {
                if let Some(i) = self.add_bridge(new_bridge, &new_faces_idx)? {
                    new_bridges_idx.set(i);
                } else {
                    ok = false;
                    break;
                }
            }

            if ok && self.search()? {
                return Ok(true)
            } else {
                self.bridges_rem = std::cmp::min(self.bridges_rem, self.bridges.count());
            }
            self.h.del_edge(start, end);
            self.h.del_node(end);
            self.h_nodes.clear(end);
            self.embedding.remove_edge(start, end);

            for j in &new_bridges_idx {
                self.remove_bridge(j);
            }
        } else {
            for face_i in &old_admissible_faces {
                //dbg!(old_admissible_faces);
                //dbg!(face_i);
                //dbg!(bridge);
                //dbg!(self.embedding.face(self.faces[*face_i]).collect::<Vec<_>>());
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
                        let old_h_nodes = self.h_nodes.clone();

                        let new_faces = self.embedding.embed_bisecting_path_after(&path, u, v);
                        self.h.union(&G::from_path(&path));
                        self.h_nodes = self.h.nodes();

                        let mut new_faces_idx = SM::Output::new();
                        new_faces_idx.set(self.faces.push(new_faces[0])?);
                        new_faces_idx.set(self.faces.push(new_faces[1])?);

                        for idx in &new_faces_idx {
                            self.admissible_bridges.insert(idx, SM::Output::new());
                        }

                        let mut ok = true;

                        for bridge_j in &old_admissible_bridges {
                            for face_j in &new_faces_idx {
                                self.update_faces(bridge_j, face_j);
                            }
                            if self.admissible_faces[bridge_j].is_empty() {
                                ok = false;
                                break
                            } else if self.admissible_faces[bridge_j].count() == 1 {
                                self.one_admissible.set(bridge_j);
                            }
                        }

                        let mut new_bridges_idx = SM::Output::new();

                        if ok {
                            for new_bridge in compute_bridges(&bridge, &self.h.clone(), &self.h_nodes.clone()) {
                                if let Some(i) = self.add_bridge(new_bridge, &new_faces_idx)? {
                                    new_bridges_idx.set(i);
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                        }

                        if ok && self.search()? {
                            return Ok(true)
                        } else {
                            self.bridges_rem = std::cmp::min(self.bridges_rem, self.bridges.count());
                        }

                        self.h = oldh;
                        self.h_nodes = old_h_nodes;
                        //dbg!(&path);
                        for (u, v) in G::from_path(&path).edges() {
                            self.embedding.remove_edge(u, v);
                        }

                        for j in &new_faces_idx {
                            //dbg!("remove", j);
                            self.remove_face(j);
                        }
                        for j in &new_bridges_idx {
                            self.remove_bridge(j);
                        }
                    }
                }
                //dbg!("insert", face_i);
                self.faces.insert(face_i, face);
                self.admissible_bridges.insert(face_i, old_admissible_bridges);
                for bridge_j in &self.admissible_bridges[face_i] {
                    self.admissible_faces[bridge_j].set(face_i);
                }
            }
        }
        self.bridges.insert(bridge_i, bridge);
        self.admissible_faces.insert(bridge_i, old_admissible_faces);
        if self.admissible_faces[bridge_i].count() == 1 {
            self.one_admissible.set(bridge_i);
        }

        for j in &self.admissible_faces[bridge_i] {
            self.admissible_bridges[j].set(bridge_i);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{subgraphs, Graph16};

    fn test_is_toroidal(graph: &Graph16) {
        let embedding = find_embedding(graph);
        assert!(embedding.is_some(), "Graph: {}", crate::parse::to_graph6(graph));
        assert!(embedding.as_ref().unwrap().genus() <= 1);
        assert_eq!(graph.to_canonical(), embedding.unwrap().to_graph().to_canonical());
    }

    #[test]
    fn check_known_obstructions() {
        let graphs: &[&str] = &[
            "9 111000011100001100001000011111111111",
            "9 000001110000111000111111111111111000",
            "15 111000000000000000000101000000001001000010000000011100010000000000100100000100011000010010101000000001111",
            "15 111000000000000000000110000001010000000101000000000001000001001000100100000100000001000010001100101111000"
        ];
        for s in graphs {
            let graph: Graph16 = crate::parse::from_upper_tri(s).unwrap();
            assert!(find_embedding(&graph).is_none(), "Graph: {}", crate::parse::to_graph6(&graph));
            for minor in subgraphs(&graph) {
                test_is_toroidal(&minor);
            }
        }
    }

    #[test]
    fn k4_plus_k5_is_toroidal() {
        let mut graph = Graph16::empty();

        for i in 0..9 {
            graph.add_node(i);
        }

        for u in 0..4 {
            for v in 0..u {
                graph.add_edge(u, v);
            }
        }

        for u in 4..9 {
            for v in 4..u {
                graph.add_edge(u, v);
            }
        }

        test_is_toroidal(&graph);
    }

}
