use std::collections::HashSet;
use crate::Graph16;
use crate::map::Map64;
use crate::embedding::*;
use crate::bitset::Bitset;

#[inline(always)]
fn compute_bridges<'a>(graph: &'a Graph16, h: &'a Graph16) -> impl 'a + Iterator<Item = Graph16> {
    graph.edges_from_to(h.nodes(), h.nodes()).filter(move |(u, v)| {
        !h.has_edge(*u, *v)
    })
    //graph.difference(h).subgraph(h.nodes()).edges()
    .map(|(u, v)| {
        let mut g = Graph16::new(0);
        g.add_node(u);
        g.add_node(v);
        g.add_edge(u, v);
        g
    }).chain(graph.subgraph(h.nodes().invert()).components()
        .map(move |mut c| {
            /*
            for (u, v) in graph.edges_from_to(c.nodes(), h.nodes()) {
                c = c.add_node(v).add_edge(u, v);
            }
            c
            */
            c.union(&graph.neighbouring(c.nodes()).bipartite_split(c.nodes(), h.nodes()));
            c
        }))
}

pub fn fastdmp(graph: &Graph16) -> Option<RotationSystem16> {
    let node_count = graph.nodes().count();
    let edge_count = graph.edges().count();

    if node_count >= 3 && edge_count + 6 > 3*node_count {
        return None
    }
    let mut h = if let Some(c) = graph.cycle() {
        c
    } else {
        // No cycle, graph must be a tree. Any embedding should be valid
        return Some(RotationSystem16::simple(graph))
    };
    
    let mut embedding = RotationSystem16::simple(&h);

    let mut faces = Map64::new();
    let mut admissible_faces = Map64::new(); //[Bitset16::new(); 16];
    let mut admissible_bridges = Map64::new(); // [HashSet<usize>; 16] = Default::default();
    let mut one_admissible = HashSet::new();

    let mut bridges = compute_bridges(graph, &h)
        //.map(|bridge| bridge.nodes())
       .collect::<Map64<_>>();
    //let mut bridges = list_bridges(graph, &h);

    //dbg!(graph, h, &bridges);

    for (i, _) in bridges.iter() {
        admissible_faces.insert(i, HashSet::new());
    }

    for face in embedding.faces() {
        let face_nodes = embedding.face_nodes(face);
        let i = faces.push(face);
        admissible_bridges.insert(i, HashSet::new());

        for (j, bridge) in bridges.iter() {
            let attachments = h.nodes().intersection(&bridge.nodes());

            if face_nodes.is_superset(&attachments) {
                admissible_faces[j].insert(i);
                admissible_bridges[i].insert(j);
            }
        }
    }

    for (i, _) in bridges.iter() {
        if admissible_faces[i].is_empty() {
            // FIXME: can this happen at this point?
            // A bridge has no addmissible faces, no embedding is possible
            return None
        } else if admissible_faces[i].len() == 1 {
            one_admissible.insert(i);
        }
    }

    loop {
        //dbg!(&embedding);
        //dbg!(&bridges);
        //dbg!(&faces);
        //dbg!(&admissible_faces);
        //dbg!(&one_admissible);
        //dbg!(embedding.faces().map(|face| (face, embedding.face_nodes(face))).collect::<Vec<_>>());
        //dbg!(&admissible_bridges);
        let (i, bridge) = if let Some(i) = one_admissible.iter().next().cloned() {
            one_admissible.remove(&i);
            (i, bridges.take(i).unwrap())
        } else if let Some(bridge) = bridges.pop() {
            bridge
        } else {
            return Some(embedding)
        };

        let bridge_nodes = bridge.nodes();
        let mut attachments = bridge_nodes.intersection(&h.nodes());

        /*
        let mut bridge = graph.subgraph(bridge_nodes);
        for u in attachments {
            bridge = bridge.del_edges(u, h.siblings(u));
        }
        */

        let old_admissible_faces = admissible_faces.take(i).unwrap();
        //admissible_faces[i] = Bitset16::new();

        for face in &old_admissible_faces {
            admissible_bridges[*face].remove(&i);
        }

        //dbg!(bridge);
        //dbg!(attachments);

        // If we only have one attachment there is no way to generate a 
        // bisecting path. Instead we add one and one edge.
        // TODO: look into ways to optimize some of this
        if attachments.count() == 1 {
            let u = attachments.smallest().unwrap();
            let v = bridge.siblings(u).smallest().unwrap();
            embedding.embed_free_edge(u, v);
            h.add_node(v);
            h.add_edge(u, v);

            /*
            if bridge.siblings(u).count() == 1 {
                // If this node had only one edge then it is no longer part
                // of bridge.
                bridge_nodes.clear(u);
                bridge = bridge.del_node(u);
            }
            if bridge.siblings(v).count() <= 1 {
                bridge_nodes.clear(v);
                bridge = bridge.del_node(v);
            }
            if bridge_nodes.is_empty() {
                continue
            }
            // We now put the (possibly modified) bridge back into place and
            // update pointers to it.
            */

            let new_bridges = compute_bridges(&bridge, &h);
                //.map(|bridge| bridge.nodes());

            for new_bridge in new_bridges {
                //dbg!(new_bridge);
                let j = bridges.push(new_bridge);
                let attachments = h.nodes().intersection(&new_bridge.nodes());
                //dbg!(attachments);
                admissible_faces.insert(j, HashSet::new());

                for face_j in &old_admissible_faces {
                    let face = faces[*face_j];
                    //dbg!(face_j);
                    //dbg!(embedding.face_nodes(face));
                    if embedding.face_nodes(face).is_superset(&attachments) {
                        admissible_faces[j].insert(*face_j);
                        admissible_bridges[*face_j].insert(j);
                    }
                }
                //dbg!(admissible_faces[j]);
                if admissible_faces[j].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return None
                }
                if admissible_faces[j].len() == 1 {
                    one_admissible.insert(j);
                } else {
                    one_admissible.remove(&j);
                }

            }

            /*
            let j = bridges.push(bridge_nodes);
            let attachments = h.nodes().intersection(&bridge_nodes);
            for face in old_admissible_faces {
                if embedding.face_nodes(faces[face]).is_superset(&attachments) {
                    admissible_bridges[face].set(j);
                    admissible_faces[j].set(face);
                }
            }

            if admissible_faces[j].is_empty() {
                // FIXME: Why is this happening?
                return None
            }

            if admissible_faces[j].count() == 1 {
                one_admissible.push(j);
            }
            */
        } else {
            let start = attachments.smallest().unwrap();
            attachments.clear(start);

            let path = bridge.path(start, attachments).unwrap();
            //dbg!(path);

            let face_i = *old_admissible_faces.iter().next().unwrap();
            let face = faces.take(face_i).unwrap();

            //dbg!(face_i);
            //dbg!(embedding.face_nodes(face));
            //dbg!(embedding.face(face).collect::<Vec<_>>());

            let new_faces = embedding.embed_bisecting_path(face, &path);
            h.union(&Graph16::from_path(&path));

            //dbg!(&embedding);

            //dbg!(embedding.face(new_faces[0]).collect::<Vec<_>>());
            //dbg!(embedding.face(new_faces[1]).collect::<Vec<_>>());

            let old_admissible_bridges = admissible_bridges.take(face_i).unwrap();

            //admissible_bridges[face_i] = Bitset16::new();

            let mut new_faces_idx = [0; 2];
            new_faces_idx[0] = faces.push(new_faces[0]);
            new_faces_idx[1] = faces.push(new_faces[1]);
            for j in &new_faces_idx {
                admissible_bridges.insert(*j, HashSet::new());
            }

            for bridge in old_admissible_bridges {
                admissible_faces[bridge].remove(&face_i);

                let attachments = h.nodes().intersection(&bridges[bridge].nodes());
    
                for face_j in &new_faces_idx {
                    if embedding.face_nodes(faces[*face_j]).is_superset(&attachments) {
                        admissible_faces[bridge].insert(*face_j);
                        admissible_bridges[*face_j].insert(bridge);
                    }
                }

                if admissible_faces[bridge].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return None
                }
                if admissible_faces[bridge].len() == 1 {
                    // FIXME: make sure we arent pushing here too many times
                    one_admissible.insert(bridge);
                } else {
                    one_admissible.remove(&bridge);
                }
            }

            // TODO: we know the changes, can this be faster?
            let new_bridges = compute_bridges(&bridge, &h);
                //.map(|bridge| dbg!(bridge).nodes());

            for new_bridge in new_bridges {
                //dbg!(new_bridge);
                let j = bridges.push(new_bridge);
                let attachments = h.nodes().intersection(&new_bridge.nodes());
                //dbg!(attachments);
                admissible_faces.insert(j, HashSet::new());

                for face_j in &new_faces_idx {
                    let face = faces[*face_j];
                    //dbg!(face_j);
                    //dbg!(embedding.face_nodes(face));
                    if embedding.face_nodes(face).is_superset(&attachments) {
                        admissible_faces[j].insert(*face_j);
                        admissible_bridges[*face_j].insert(j);
                    }
                }
                //dbg!(admissible_faces[j]);
                if admissible_faces[j].is_empty() {
                    // Bridge with no admissible faces, there is no embedding
                    return None
                }
                if admissible_faces[j].len() == 1 {
                    one_admissible.insert(j);
                } else {
                    one_admissible.remove(&j);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k4_planar() {
        let graph = Graph16::regular(4);
        assert!(fastdmp(&graph).is_some());
    }

    #[test]
    fn k5_not_planar() {
        let graph = Graph16::regular(5);
        assert!(fastdmp(&graph).is_none());
    }

    #[test]
    fn k5_minors_planar() {
        for graph in Graph16::regular(5).minors() {
            assert!(fastdmp(&graph).is_some());
        }
    }
}
