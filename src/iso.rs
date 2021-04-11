use std::collections::HashSet;
use std::hash::Hash;
use crate::graph::{Graph, Coloring, Graph16, Coloring16};
use crate::permutation::{Permutation, Perm16};
use crate::seq::{Seq, Seq16};
use crate::bitset::{Intset, Bitset, Bitset64};

pub fn refine<G: Graph>(graph: &G, mut coloring: G::Coloring, seq: G::Path) -> G::Coloring {
    let mut cell_set = Bitset64::new();
    for x in seq.iter() {
        cell_set.set(x);
    }

    while let Some(w) = cell_set.smallest() {
        if coloring.discrete() {
            return coloring
        }
        cell_set.clear(w);

        let w_cell = coloring.get_cell(w);

        for x in coloring.cells().iter() {
            let mut frags = [Bitset64::new(); 65];
            let x_cell = coloring.get_cell(x);

            for u in x_cell.iter() {
                let w_edges = graph.siblings(u).intersection(&w_cell).count();
                frags[w_edges].set(u);
            }
            
            let mut x_set = Bitset64::new();
            x_set.set(x);

            let col_start = coloring.next_color();
            let mut col = col_start;
            let mut iter = frags.iter().enumerate()
                .filter(|(_, frag)| !frag.is_empty());
            iter.next();
            for (_, frag) in iter {
                for u in frag.iter() {
                    coloring.set(u, col);
                }
                cell_set.set(col as usize);
                col += 1;
            }
        }
    }

    coloring
}

pub struct SearchResults<G: Graph> {
    pub automorphisms: HashSet<G::Perm>,
    pub canonical_relabeling: G::Perm,
    pub canonical_graph: G,
    pub orbits: Seq16,
}

pub fn search_tree<G: Graph + Ord>(graph: G) -> SearchResults<G>
    where G::Perm: Eq + Hash,
          G::Path: Eq + Hash
{
    let mut coloring = G::Coloring::new();

    for u in graph.nodes().iter() {
        coloring.set(u, 0);
    }

    let mut tree = SearchTree::new(graph);
    tree.start_search(coloring);

    let canonical = tree.largest_invariant.unwrap();

    SearchResults {
        automorphisms: tree.automorphisms,
        canonical_relabeling: canonical.0,
        canonical_graph: canonical.1.end_graph.unwrap(),
        orbits: tree.orbits,
    }
}

pub struct SearchTree<G: Graph> {
    graph: G,
    automorphisms: HashSet<G::Perm>,
    autonodes: HashSet<G::Path>,
    largest_invariant: Option<(G::Perm, NodeInvariant<G>)>,
    first_node: Option<(G::Perm, NodeInvariant<G>)>,
    orbits: Seq16,
    pub auto_prune: bool,
}

impl<G: Graph + Ord> SearchTree<G>
    where G::Perm: Eq + Hash,
          G::Path: Eq + Hash
{
    pub fn new(graph: G) -> SearchTree<G> {
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

    fn search(&mut self, coloring: G::Coloring, mut invariant: NodeInvariant<G>, seq: G::Path) {
        if let Some(perm) = coloring.permutation() {
            invariant.add_node(&coloring);
            invariant.add_leaf(self.graph.clone(), &perm);


            if let Some((ref first_perm, ref first_invariant)) = &self.first_node {
                if first_invariant == &invariant {
                    let res = perm.chain(&first_perm.invert());
                    //let res = first_perm.invert().chain(&perm);

                    // Check that we have an automorphism
                    debug_assert!(self.graph == {
                        let mut graph = self.graph.clone();
                        graph.shuffle(&res);
                        graph
                    });
                    self.automorphisms.insert(res);
                }
            }

            if let Some((ref first_perm, ref first_invariant)) = &self.largest_invariant {
                if first_invariant == &invariant {
                    let res = perm.chain(&first_perm.invert());
                    //let res = first_perm.invert().chain(&perm);

                    // Check that we have an automorphism
                    debug_assert!(self.graph == {
                        let mut graph = self.graph.clone();
                        graph.shuffle(&res);
                        graph
                    });
                    self.automorphisms.insert(res);
                }
            }


            let inv_greater = self.largest_invariant.as_ref().map(|(_, largest)| 
                invariant.cmp_prefix(largest) == std::cmp::Ordering::Greater)
                .unwrap_or(true);

            if inv_greater {
                self.largest_invariant = Some((perm.clone(), invariant.clone()));
            }
            if self.first_node.is_none() {
                self.first_node = Some((perm, invariant));
            }
        } else {
            invariant.add_node(&coloring);

            // TODO: faster without invariant, find a better one
            let cell = coloring.cells().iter()
                .map(|col| coloring.get_cell(col))
                .filter(|cell| cell.count() > 1)
                .next().unwrap();

            for u in cell.iter() {
                let mut seq = seq.clone();
                seq.push(u);

                if self.auto_prune && self.autonodes.contains(&seq) {
                    continue
                } else {
                    let mut pruned = false;
                    let mut autonodes = Vec::new();
                    for perm in &self.automorphisms {
                        let mut autonode = G::Path::new();
                        for i in seq.iter() {
                            autonode.push(perm.get(i));
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
                let mut refined = coloring.clone();
                refined.individualize(u);

                let mut cells = G::Path::new();
                cells.push(coloring.get(u) as usize);
                let refined = refine(&self.graph, refined, cells);
                self.search(refined, invariant.clone(), seq);
            }
        }
    }

    fn start_search(&mut self, coloring: G::Coloring) {
        let mut cells = G::Path::new();
        for cell in coloring.cells().iter() {
            cells.push(cell);
        }

        let refined = refine(&self.graph, coloring, cells);
        self.search(refined, NodeInvariant::new(), G::Path::new());
    }

    /*
    pub fn find_canonical(&mut self) -> G {
        let mut coloring = G::Coloring::new();

        for u in self.graph.nodes().iter() {
            coloring.set(u, 0);
        }

        self.start_search(coloring);

        self.largest_invariant.unwrap().1.end_graph.unwrap()
    }
    */
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct NodeInvariant<G: Graph> {
    seq: G::Path,
    end_graph: Option<G>,
}

impl<G: Graph + Ord> NodeInvariant<G> {
    fn new() -> Self {
        Self {
            seq: G::Path::new(),
            end_graph: None,
        }
    }

    fn cmp_prefix(&self, other: &Self) -> std::cmp::Ordering {
        let seq_cmp = self.seq.iter().cmp(other.seq.iter());

        match (&self.end_graph, &other.end_graph) {
            (Some(ref ga), Some(ref gb)) => seq_cmp.then(ga.cmp(gb)),
            _ => seq_cmp,
        }
    }

    fn add_node(&mut self, _coloring: &G::Coloring) -> u8 {
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

    fn add_leaf(&mut self, mut graph: G, permutation: &G::Perm) {
        graph.shuffle(permutation);
        self.end_graph = Some(graph)
    }
}
