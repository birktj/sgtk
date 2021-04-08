use std::collections::HashSet;
use crate::graph::{Graph16, Coloring16};
use crate::seq::Seq16;
use crate::bitset::{Bitset, Bitset16};

pub fn refine(graph: &Graph16, mut coloring: Coloring16, seq: Seq16) -> Coloring16 {
    let mut cell_set = Bitset16::new();
    for x in seq.iter() {
        cell_set.set(*x as usize);
    }

    while let Some(w) = cell_set.smallest() {
        if coloring.discrete() {
            return coloring
        }
        cell_set.clear(w);

        let w_cell = coloring.get_cell(w as u8);

        for x in coloring.cells() {
            let mut frags = [Bitset16::new(); 17];
            let x_cell = coloring.get_cell(x as u8);

            for u in x_cell.into_iter() {
                let w_edges = graph.siblings(u).intersection(&w_cell).count();
                frags[w_edges].set(u);
            }
            
            let mut x_set = Bitset16::new();
            x_set.set(x);

            let col_start = coloring.next_color();
            let mut col = col_start;
            let mut iter = frags.iter().enumerate()
                .filter(|(_, frag)| !frag.is_empty());
            iter.next();
            for (_, frag) in iter {
                for u in *frag {
                    coloring.set(u, col);
                }
                cell_set.set(col as usize);
                col += 1;
            }
        }
    }

    coloring
}

pub fn individualize(mut coloring: Coloring16, u: usize) -> Coloring16 {
    let cu = coloring.get(u);
    for v in 0..16 {
        if coloring.defined(v) {
            let cv = coloring.get(v);
            if u != v && cv >= cu {
                coloring.set(v, cv+1);
            }
        }
    }
    coloring
}

pub struct SearchResults {
    pub automorphisms: HashSet<Seq16>,
    pub canonical_relabeling: Seq16,
    pub canonical_graph: Graph16,
    pub orbits: Seq16,
}

pub fn search_tree(graph: Graph16) -> SearchResults {
    let mut coloring = Coloring16::new();

    for u in graph.nodes() {
        coloring.set(u, 0);
    }

    let mut tree = SearchTree::new(graph);
    tree.start_search(coloring);

    SearchResults {
        automorphisms: tree.automorphisms,
        canonical_relabeling: tree.largest_invariant.unwrap().0,
        canonical_graph: tree.largest_invariant.unwrap().1.end_graph.unwrap(),
        orbits: tree.orbits,
    }
}

pub struct SearchTree {
    graph: Graph16,
    automorphisms: HashSet<Seq16>,
    autonodes: HashSet<Seq16>,
    largest_invariant: Option<(Seq16, NodeInvariant)>,
    first_node: Option<(Seq16, NodeInvariant)>,
    orbits: Seq16,
    pub auto_prune: bool,
}

impl SearchTree {
    pub fn new(graph: Graph16) -> SearchTree {
        let mut orbits = Seq16::new();
        for i in 0..16 {
            orbits.push(i);
        }
        SearchTree {
            graph,
            automorphisms: HashSet::new(),
            autonodes: HashSet::new(),
            largest_invariant: None,
            first_node: None,
            orbits,
            auto_prune: true,
        }
    }

    #[inline(never)]
    fn search(&mut self, coloring: Coloring16, mut invariant: NodeInvariant, seq: Seq16) {
        if let Some(perm) = coloring.permutation() {
            invariant.add_node(&coloring);
            invariant.add_leaf(self.graph, &perm);


            if let Some((ref first_perm, ref first_invariant)) = &self.first_node {
                if first_invariant == &invariant {
                    let mut res = Seq16::from_slice(&[0u8; 16] as &[_]);
                    let mut first_inv = Seq16::from_slice(&[0u8; 16] as &[_]);

                    for (i, j) in first_perm.iter().enumerate() {
                        first_inv[*j as usize] = i as u8;
                    }
                    for i in 0..16 {
                        res[i] = first_inv[perm[i] as usize];
                    }

                    // Check that we have an automorphism
                    //debug_assert!(self.graph == self.graph.shuffle2(&res));
                    self.automorphisms.insert(res);
                }
            }

            if let Some((ref first_perm, ref first_invariant)) = &self.largest_invariant {
                if first_invariant == &invariant {
                    let mut res = Seq16::from_slice(&[0u8; 16] as &[_]);
                    let mut first_inv = Seq16::from_slice(&[0u8; 16] as &[_]);

                    for (i, j) in first_perm.iter().enumerate() {
                        first_inv[*j as usize] = i as u8;
                    }
                    for i in 0..16 {
                        res[i] = first_inv[perm[i] as usize];
                    }

                    // Check that we have an automorphism
                    //debug_assert!(self.graph == self.graph.shuffle2(&res));
                    self.automorphisms.insert(res);
                }
            }


            let inv_greater = self.largest_invariant.as_ref().map(|(_, largest)| 
                invariant.cmp_prefix(largest) == std::cmp::Ordering::Greater)
                .unwrap_or(true);

            if inv_greater {
                self.largest_invariant = Some((perm, invariant));
            }
            if self.first_node.is_none() {
                self.first_node = Some((perm, invariant));
            }
        } else {
            invariant.add_node(&coloring);

            // TODO: faster without invariant, find a better one
            
            let cell = coloring.cells().into_iter()
                .map(|col| coloring.get_cell(col as u8))
                .filter(|cell| cell.count() > 1)
                .next().unwrap();

            for u in cell {
                let mut seq = seq;
                seq.push(u);

                if self.auto_prune && self.autonodes.contains(&seq) {
                    continue
                } else {
                    let mut pruned = false;
                    let mut autonodes = Vec::new();
                    for perm in &self.automorphisms {
                        let mut autonode = Seq16::new();
                        for i in seq.iter() {
                            autonode.push(perm[*i as usize] as usize);
                            self.orbits[*i as usize] = std::cmp::min(self.orbits[*i as usize], perm[*i as usize]);
                        }
                        if self.autonodes.contains(&autonode) {
                            pruned = true;
                            break
                        }
                        autonodes.push(autonode);
                    }

                    if self.auto_prune && pruned {
                        continue
                    }
                    for autonode in autonodes {
                        self.autonodes.insert(autonode);
                    }
                }

                let refined = individualize(coloring, u);
                let mut cells = Seq16::new();
                cells.push(coloring.get(u) as usize);
                let refined = refine(&self.graph, refined, cells);
                self.search(refined, invariant, seq);
            }
        }
    }

    fn start_search(&mut self, coloring: Coloring16) {
        let mut cells = Seq16::new();
        for cell in coloring.cells() {
            cells.push(cell);
        }

        let refined = refine(&self.graph, coloring, cells);
        self.search(refined, NodeInvariant::new(), Seq16::new());
    }

    pub fn find_canonical(&mut self) -> Graph16 {
        let mut coloring = Coloring16::new();

        for u in self.graph.nodes() {
            coloring.set(u, 0);
        }

        self.start_search(coloring);

        self.largest_invariant.unwrap().1.end_graph.unwrap()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct NodeInvariant {
    seq: Seq16,
    end_graph: Option<Graph16>,
}

impl NodeInvariant {
    fn new() -> NodeInvariant {
        NodeInvariant {
            seq: Seq16::new(),
            end_graph: None,
        }
    }

    fn cmp_prefix(&self, other: &NodeInvariant) -> std::cmp::Ordering {
        let len = std::cmp::min(self.seq.len(), other.seq.len());
        let seq_cmp = (&self.seq.slice()[0..len]).cmp(&other.seq.slice()[0..len]);

        match (&self.end_graph, &other.end_graph) {
            (Some(ref ga), Some(ref gb)) => seq_cmp.then(ga.cmp(gb)),
            _ => seq_cmp,
        }
    }

    fn add_node(&mut self, _coloring: &Coloring16) -> u8 {
        // FIXME: using node invariants makes code slower, not faster
        /*
        use std::hash::{Hash, Hasher};
        // TODO: is this a valid invariant
        // TODO: better invariant?
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for cell in coloring.cells() {
            //coloring.get_cell(cell as u8).hash(&mut hasher);
            //cell.hash(&mut hasher);
            coloring.get_cell(cell as u8).count().hash(&mut hasher);
            /*
            let mut adj_colors = Bitset16::new();
            for u in coloring.get_cell(cell as u8) {
                adj_colors.set(coloring.get(u) as usize);
            }
            adj_colors.hash(&mut hasher);*/
        }
        let val = (hasher.finish() & 0xff) as u8;
        self.seq.push(val as usize);
        //dbg!(&self.seq);
        val
        */
        0
    }

    fn add_leaf(&mut self, mut graph: Graph16, permutation: &Seq16) {
        graph.shuffle2(permutation);
        self.end_graph = Some(graph)
    }
}
