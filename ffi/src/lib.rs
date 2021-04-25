use sgtk::prelude::*;
use sgtk::graph::Graph16;
use sgtk::bitset::Bitset16;
use sgtk::embedding::RotationSystem16;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};

fn graph16_from_pointer(raw: *const u16) -> Graph16 {
    let raw = unsafe { std::slice::from_raw_parts(raw, 16) };
    let mut graph = [sgtk::bitset::Bitset16::new(); 16];
    for (i, u) in raw.iter().enumerate() {
        graph[i] = sgtk::bitset::Bitset16::from_u16(*u);
    }
    sgtk::graph::Graph16::from_raw(&graph)
}

fn graph16_to_pointer(graph: &Graph16, out: *mut u16) {
    let raw = graph.to_raw();
    let out = unsafe { std::slice::from_raw_parts_mut(out, 16) };
    for (i, v) in raw.iter().enumerate() {
        out[i] = v.to_u16();
    }
}

struct LevelData {
    embeddings: [Option<RotationSystem16>; 16],
    subgraphs: [Option<Graph16>; 16],
    subgraphs2: Vec<HashMap<Bitset16, Graph16>>,
    siblings: Vec<HashSet<Bitset16>>,
    num_total: u64,
    num_toroidal: u64,
    num_non_toroidal: u64,
    num_prev_embedding: u64,
    num_calls: u64,
    num_superset: u64,
    num_kuratowski: u64,
    num_check_obs: u64,
    num_k5: u64,
}

impl LevelData {
    fn new() -> Self {
        Self {
            embeddings: [None; 16],
            subgraphs: [None; 16],
            subgraphs2: vec![HashMap::new(); 16],
            siblings: vec![HashSet::new(); 16],
            num_total: 0,
            num_toroidal: 0,
            num_non_toroidal: 0,
            num_prev_embedding: 0,
            num_superset: 0,
            num_calls: 0,
            num_kuratowski: 0,
            num_check_obs: 0,
            num_k5: 0,
        }
    }
}

fn is_obstruction(graph: &Graph16, k: Graph16) -> bool {
    for u in graph.nodes().iter() {
        if graph.siblings(u).count() < 3 {
            return false
        }
    }

    for (u, v) in graph.edges() {
        let mut subgraph = graph.clone();
        subgraph.del_edge(u, v);
        let mut embedder = sgtk::toroidal::Embedder::new();
        embedder.add_subgraph(k.clone());
        if embedder.find_embedding(graph).embedding.is_none() {
            return false
        }
    }
    true
}

fn is_k33(graph: &Graph16) -> bool {
    let mut n = 0;
    for u in graph.nodes() {
        let count = graph.siblings(u).count();
        if count > 3 {
            return false
        } else if count == 3 {
            n += 1;
        }
    }

    n == 6
}

thread_local! {
    static LEVEL_DATA: RefCell<LevelData> = RefCell::new(LevelData::new());
}

#[no_mangle]
pub extern "C" fn sgtk_graph16_prune_toroidal(n: u32, maxn: u32, graph: *const u16) -> i32 {
    let n = n as usize;
    let maxn = maxn as usize;

    let graph = graph16_from_pointer(graph);

    LEVEL_DATA.with(|level_data| {
        if level_data.borrow().num_total % 1000000 == 0 {
            dbg!(level_data.borrow().num_total);
            dbg!(level_data.borrow().num_calls);
            dbg!(level_data.borrow().num_toroidal);
            dbg!(level_data.borrow().num_non_toroidal);
            dbg!(level_data.borrow().num_prev_embedding);
            dbg!(level_data.borrow().num_superset);
            dbg!(level_data.borrow().num_kuratowski);
            dbg!(level_data.borrow().num_check_obs);
            dbg!(level_data.borrow().num_k5);
        }
        let siblings = graph.siblings(n-1);
        level_data.borrow_mut().num_total += 1;
        let k = level_data.borrow().subgraphs[n-1].clone();
        let mut k = k.or_else(|| level_data.borrow().subgraphs[n].clone()
            .filter(|k| graph.is_supergraph(&k)));
    
        if k.is_none() {
            for u in siblings {
                let mut siblings = siblings.clone();
                siblings.clear(u);
                if let Some(subgraph) = level_data.borrow().subgraphs2[n].get(&siblings) {
                    k = Some(subgraph.clone());
                    break
                }
            }
        }

        level_data.borrow_mut().subgraphs[n] = k.clone();
        level_data.borrow_mut().subgraphs2[n+1] = HashMap::new();
        if let Some(subgraph) = &k {
            level_data.borrow_mut().subgraphs2[n].insert(siblings.clone(), subgraph.clone());
        }

        level_data.borrow_mut().siblings[n+1] = HashSet::new();
        level_data.borrow_mut().embeddings[n] = None;

        /*
        if !graph.is_connected() {
            return 0
        }
        */

        if n < maxn && k.is_none() {
            if let Some(embedding) = sgtk::planar::find_embedding(&graph) {
                level_data.borrow_mut().embeddings[n] = Some(embedding);
                level_data.borrow_mut().num_toroidal += 1;
                return 0
            }

            k = k.or_else(|| {
                    level_data.borrow_mut().num_kuratowski += 1;
                    Some(sgtk::toroidal::find_kuratowski(graph))
                });

            if is_k33(k.as_ref().unwrap()) {
                level_data.borrow_mut().subgraphs[n] = k.clone();
            }
        }

        let mut found_superset = false;
        for u in siblings {
            let mut siblings = siblings.clone();
            siblings.clear(u);
            if level_data.borrow().siblings[n].contains(&siblings) {
                found_superset = true;
                break
            }
        }
        if found_superset {
            level_data.borrow_mut().siblings[n].insert(siblings);
            level_data.borrow_mut().num_superset += 1;
            level_data.borrow_mut().num_non_toroidal += 1;
            return 1
        }

        let mut found_embedding = None;
        if let Some(embedding) = level_data.borrow().embeddings[n-1].as_ref() {
            for face in embedding.faces() {
                if embedding.face_nodes(face).is_superset(&siblings) {
                    let mut siblings = siblings.clone();
                    let mut new_embedding = embedding.clone();
                    for (u, v) in embedding.face(face) {
                        if siblings.get(u) {
                            // FIXME: this is incorrect, order around n-1 is wrong
                            new_embedding.embed_edge_before(u, v, n-1);
                            siblings.clear(u);
                        }
                    }
                    found_embedding = Some(new_embedding);
                }
            }
        }

        if let Some(_embedding) = found_embedding {
            level_data.borrow_mut().num_prev_embedding += 1;
            level_data.borrow_mut().num_toroidal += 1;
            // TODO: see above, must be fixed before we can do this
            //level_data.borrow_mut().embeddings[n] = Some(embedding);
            return 0
        }

        if k.is_none() {
            if let Some(embedding) = sgtk::planar::find_embedding(&graph) {
                level_data.borrow_mut().embeddings[n] = Some(embedding);
                level_data.borrow_mut().num_toroidal += 1;
                return 0
            }
        }

        let k = k.unwrap_or_else(|| {
                level_data.borrow_mut().num_kuratowski += 1;
                sgtk::toroidal::find_kuratowski(graph)
            });

        if is_k33(&k) {
            level_data.borrow_mut().subgraphs[n] = Some(k);
        } else {
            level_data.borrow_mut().num_k5 += 1;
        }

        level_data.borrow_mut().num_calls += 1;
        let mut embedder = sgtk::toroidal::Embedder::new();
        embedder.add_subgraph(k);
        if let Some(mut embedding) = embedder.find_embedding(&graph).embedding {
            level_data.borrow_mut().embeddings[n] = Some(embedding.clone());
            let mut graph = graph.clone();
            graph.del_node(n-1);
            embedding.remove_node(n-1);
            if graph.is_connected() {
                level_data.borrow_mut().embeddings[n-1] = Some(embedding);
            }
            level_data.borrow_mut().num_toroidal += 1;
            0
        } else {
            /*
            level_data.borrow_mut().num_check_obs += 1;
            if is_obstruction(&graph, k.clone()) {
                println!("{}", sgtk::parse::to_graph6(&graph));
            }
            */
            level_data.borrow_mut().num_non_toroidal += 1;
            level_data.borrow_mut().siblings[n].insert(siblings);
            1
        }
    })
}

#[no_mangle]
pub extern "C" fn sgtk_graph16_find_kuratowski(raw: *const u16, out: *mut u16) -> i32 {
    let graph = graph16_from_pointer(raw);
    if !graph.is_connected() {
        0
    } else if sgtk::planar::fastdmp(&graph).is_some() {
        0
    } else {
        let k = sgtk::toroidal::find_kuratowski(graph);
        graph16_to_pointer(&k, out);
        1
    }
}

#[no_mangle]
pub extern "C" fn sgtk_graph16_is_k33(raw: *const u16) -> i32 {
    let graph = graph16_from_pointer(raw);
    let mut n = 0;
    for u in graph.nodes() {
        if graph.siblings(u).count() > 3 {
            return 0;
        }
        if graph.siblings(u).count() > 2 {
            n += 1;
        }
    }
    if n == 6 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn sgtk_is_graph16_planar(raw: *const u16) -> i32 {
    let graph = graph16_from_pointer(raw);
    if !graph.is_connected() || sgtk::planar::fastdmp(&graph).is_some() {
        1
    } else {
        0
    }
}

/*
#[no_mangle]
pub extern "C" fn sgtk_is_graph16_toroidal(raw: *const u16, subgraph: *const u16) -> i32 {
    let graph = graph16_from_pointer(raw);
    if !graph.is_connected() {
        1
    } else if !subgraph.is_null() {
        let subgraph = graph16_from_pointer(subgraph);
        if sgtk::toroidal::find_embedding_with_subgraph(&graph, subgraph).is_some() {
            1
        } else {
            0
        }
    } else if sgtk::toroidal::find_embedding(&graph).is_some() {
        1
    } else {
        0
    }
}
*/

#[no_mangle]
pub extern "C" fn sgtk_is_graph16_toroidal_obstruction(raw: *const u16) -> i32 {
    let graph = graph16_from_pointer(raw);

    if !graph.is_connected() {
        return 0
    }
    for u in graph.nodes().iter() {
        if graph.siblings(u).count() < 3 {
            return 0
        }
    }

    for subgraph in sgtk::graph::subgraphs(&graph).filter(|g| g.is_connected()) {
        if sgtk::toroidal::find_embedding(&subgraph).is_none() {
            return 0
        }
    }

    1
}
