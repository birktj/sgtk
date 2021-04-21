use crate::map::{Map64, DynMap, FullMapError};
use crate::bitset::{Bitset64, DynIntSet};
use crate::embedding::Face;
use crate::prelude::*;

#[inline(always)]
fn compute_bridges<'a, G: Graph>(graph: &'a G, h: &'a G, h_nodes: &'a G::Set) -> impl 'a + Iterator<Item = G> {
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

pub fn find_embedding<G: Graph>(graph: &G) -> Option<G::Embedding> {
    if graph.is_connected() {
        fastdmp(graph)
    } else {
        let mut embedding = G::Embedding::empty();
        for component in graph.to_owned().components() {
            embedding.embed_disconnected(&fastdmp(&component)?);
        }
        Some(embedding)
    }
}

pub fn fastdmp<G: Graph>(graph: &G) -> Option<G::Embedding> {
    dmp_inner::<G, Bitset64, Map64<Bitset64>, Map64<G>, Map64<Face>>(graph)
        .unwrap_or_else(|_| {
            dmp_inner::<G, DynIntSet, DynMap<DynIntSet>, DynMap<G>, DynMap<Face>>(graph)
                .unwrap()
        })
}

fn dmp_inner<G: Graph, B: Intset, SM: Slotmap<Output = B>, BM: Slotmap<Output = G>, FM: Slotmap<Output = Face>>(graph: &G) -> Result<Option<G::Embedding>, FullMapError> 
    where for<'a> &'a SM: IntoIterator<Item = (usize, &'a B)>,
          for<'a> &'a BM: IntoIterator<Item = (usize, &'a G)>,
          for<'a> &'a FM: IntoIterator<Item = (usize, &'a Face)>,
          for<'a> &'a B: IntoIterator<Item = usize>,
{
    let node_count = graph.nodes().count();
    let edge_count = graph.edges().count();

    if node_count >= 3 && edge_count + 6 > 3*node_count {
        return Ok(None)
    }
    let mut h = if let Some(c) = graph.cycle() {
        c
    } else {
        // No cycle, graph must be a tree. Any embedding should be valid
        return Ok(Some(G::Embedding::simple(graph)))
    };
    let mut h_nodes = h.nodes();
    
    let mut embedding = G::Embedding::simple(&h);

    let mut faces              = FM::new();
    let mut admissible_faces   = SM::new();
    let mut admissible_bridges = SM::new();
    let mut one_admissible     = B::new();
    let mut bridges            = BM::new();

    for bridge in compute_bridges(graph, &h, &h_nodes) {
        bridges.push(bridge)?;
    }

    for (i, _) in &bridges {
        admissible_faces.insert(i, B::new())?;
    }

    for face in embedding.faces() {
        let face_nodes = embedding.face_nodes(face);
        let i = faces.push(face)?;
        admissible_bridges.insert(i, B::new())?;

        for (j, bridge) in &bridges {
            let attachments = h.nodes().intersection(&bridge.nodes());

            if face_nodes.is_superset(&attachments) {
                admissible_faces[j].set(i);
                admissible_bridges[i].set(j);
            }
        }
    }

    for (i, _) in &bridges {
        if admissible_faces[i].is_empty() {
            // FIXME: can this happen at this point?
            // A bridge has no addmissible faces, no embedding is possible
            return Ok(None)
        } else if admissible_faces[i].count() == 1 {
            one_admissible.set(i);
        }
    }

    loop {
        let (i, bridge) = if let Some(i) = one_admissible.smallest() {
            one_admissible.clear(i);
            (i, bridges.take(i).unwrap())
        } else if let Some(bridge) = bridges.pop() {
            bridge
        } else {
            return Ok(Some(embedding))
        };

        let bridge_nodes = bridge.nodes();
        let mut attachments = bridge_nodes.intersection(&h_nodes);

        let old_admissible_faces = admissible_faces.take(i).unwrap();

        for face in &old_admissible_faces {
            admissible_bridges[face].clear(i);
        }

        // If we only have one attachment there is no way to generate a 
        // bisecting path. Instead we add one and one edge.
        // TODO: look into ways to optimize some of this
        if attachments.count() == 1 {
            let u = attachments.smallest().unwrap();
            let v = bridge.siblings(u).smallest().unwrap();
            embedding.embed_free_edge(u, v);
            h_nodes.set(v);
            h.add_node(v);
            h.add_edge(u, v);


            for new_bridge in compute_bridges(&bridge, &h, &h_nodes) {
                let attachments = h_nodes.intersection(&new_bridge.nodes());
                let j = bridges.push(new_bridge)?;
                admissible_faces.insert(j, B::new())?;

                for face_j in &old_admissible_faces {
                    let face = faces[face_j];
                    if embedding.face_nodes(face).is_superset(&attachments) {
                        admissible_faces[j].set(face_j);
                        admissible_bridges[face_j].set(j);
                    }
                }
                if admissible_faces[j].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return Ok(None)
                }
                if admissible_faces[j].count() == 1 {
                    one_admissible.set(j);
                } else {
                    one_admissible.clear(j);
                }

            }

        } else {
            let start = attachments.smallest().unwrap();
            attachments.clear(start);

            let path = bridge.path(start, &attachments).unwrap();

            let face_i = old_admissible_faces.smallest().unwrap();
            let face = faces.take(face_i).unwrap();

            let new_faces = embedding.embed_bisecting_path(face, &path);
            for i in path.iter() {
                h_nodes.set(i);
            }
            h.union(&G::from_path(&path));

            let old_admissible_bridges = admissible_bridges.take(face_i).unwrap();

            let mut new_faces_idx = [0; 2];
            new_faces_idx[0] = faces.push(new_faces[0])?;
            new_faces_idx[1] = faces.push(new_faces[1])?;
            for j in &new_faces_idx {
                admissible_bridges.insert(*j, B::new())?;
            }

            for bridge in &old_admissible_bridges {
                admissible_faces[bridge].clear(face_i);

                let attachments = h_nodes.intersection(&bridges[bridge].nodes());
    
                for face_j in &new_faces_idx {
                    if embedding.face_nodes(faces[*face_j]).is_superset(&attachments) {
                        admissible_faces[bridge].set(*face_j);
                        admissible_bridges[*face_j].set(bridge);
                    }
                }

                if admissible_faces[bridge].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return Ok(None)
                }
                if admissible_faces[bridge].count() == 1 {
                    // FIXME: make sure we arent pushing here too many times
                    one_admissible.set(bridge);
                } else {
                    one_admissible.clear(bridge);
                }
            }

            // TODO: we know the changes, can this be faster?
            for new_bridge in compute_bridges(&bridge, &h, &h_nodes) {
                let attachments = h_nodes.intersection(&new_bridge.nodes());
                let j = bridges.push(new_bridge)?;
                admissible_faces.insert(j, B::new())?;

                for face_j in &new_faces_idx {
                    let face = faces[*face_j];
                    if embedding.face_nodes(face).is_superset(&attachments) {
                        admissible_faces[j].set(*face_j);
                        admissible_bridges[*face_j].set(j);
                    }
                }
                if admissible_faces[j].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return Ok(None)
                }
                if admissible_faces[j].count() == 1 {
                    one_admissible.set(j);
                } else {
                    one_admissible.clear(j);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{minors, Graph16};

    fn test_is_planar(graph: &Graph16) {
        let embedding = find_embedding(graph);
        assert!(embedding.is_some());
        assert_eq!(embedding.as_ref().unwrap().genus(), 0);
        assert_eq!(graph.to_canonical(), embedding.unwrap().to_graph().to_canonical());
    }

    #[test]
    fn k1_planar() {
        test_is_planar(&Graph16::complete(1));
    }

    #[test]
    fn k4_planar() {
        test_is_planar(&Graph16::complete(4));
    }

    #[test]
    fn k5_not_planar() {
        let graph = Graph16::complete(5);
        assert!(find_embedding(&graph).is_none());
    }

    #[test]
    fn k5_minors_planar() {
        for graph in minors(&Graph16::complete(5)) {
            test_is_planar(&graph);
        }
    }

    #[test]
    fn k1_x10_planar() {
        let mut graph = Graph16::empty();
        for u in 0..10 {
            graph.add_node(u);
        }
        test_is_planar(&graph);
    }
}
