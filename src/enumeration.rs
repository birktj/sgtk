use std::collections::HashSet;
use crate::Graph16;
use crate::seq::Seq16;
use crate::bitset::{Bitset, Bitset16};
use crate::iso::search_tree;

fn check_auto<'a>(n: usize, cut_vert: usize, auto_gens: &'a HashSet<Seq16>) -> impl 'a + Iterator<Item = Seq16> {
    let mut perm = Seq16::new();
    for i in 0..n + 1 {
        perm.push(i);
    }

    let mut seen_perms = HashSet::new();
    let mut perms = Vec::new();
    let mut gen_iter = auto_gens.into_iter();

    std::iter::from_fn(move || {
        loop {
            while let Some(gen) = gen_iter.next() {
                let mut new_perm = Seq16::new();
                for (j, i) in perm.iter().enumerate() {
                    new_perm.push(gen[*i as usize] as usize);
                }
                if !seen_perms.contains(&new_perm) {
                    seen_perms.insert(new_perm);
                    perms.push(new_perm);
                    if usize::from(new_perm[cut_vert]) == n {
                        return Some(new_perm)
                    }
                }
            }

            perm = perms.pop()?;
            gen_iter = auto_gens.into_iter();
        }
    })
}

#[derive(Debug)]
pub struct Counts {
    search_tree: usize,
    num_test: usize,
    compute_orbit: usize,
}

pub struct Enumerator16<F> {
    maxn: usize,
    prune: F,
    pub counts: Counts,
    //pub graphs: Vec<Graph16>,
    pub graphs: HashSet<Graph16>,
}

impl Enumerator16<()> {
    pub fn new(maxn: usize) -> Enumerator16<impl FnMut(&Graph16) -> bool> {
        Enumerator16 {
            prune: |_: &Graph16| false,
            maxn,
            counts: Counts {
                search_tree: 0,
                num_test: 0,
                compute_orbit: 0,
            },
            //graphs: Vec::new(),
            graphs: HashSet::new(),
        }
    }
}

impl<F> Enumerator16<F> {
    pub fn set_prune<F2: FnMut(&Graph16) -> bool>(self, prune: F2) -> Enumerator16<F2> {
        Enumerator16 {
            prune,
            maxn: self.maxn,
            counts: self.counts,
            graphs: self.graphs,
        }
    }
}

impl<F: FnMut(&Graph16) -> bool> Enumerator16<F> {
    fn enumerate_inner(&mut self, auto_gens: HashSet<Seq16>, graph: Graph16, n: usize) {
        if n >= self.maxn {
            return
        }

        //dbg!(&auto_gens);

        let mut children: Vec<i32> = vec![-1; 1 << (n+1)];
        for gen in auto_gens {
            for mut i in 0..children.len() {
                let mut k = Bitset16::from_u16(i as u16);
                k.shuffle(&gen);
                let mut k = k.to_u16() as usize;
                /*
                for j in 0..n {
                    if (1 << j) & i != 0 {
                        k += 1 << gen[j];
                    }
                }
                */
                //dbg!(i, k);
                while children[k] != -1 {
                    k = children[k] as usize;
                }
                while children[i] != -1 {
                    i = children[i] as usize;
                }
                if i != k {
                    let smaller = std::cmp::min(i, k);
                    let bigger = std::cmp::max(i, k);
                    children[bigger] = smaller as i32;
                }
            }
        }
        //dbg!(n, &children);

        let mut curr_graphs = HashSet::new();

        for edges in Bitset16::enumerate(n) {
            self.counts.num_test += 1;
            if children[edges.to_u16() as usize] != -1 {
                continue
            }
            let mut new_graph = graph;
            new_graph.add_node(n);
            new_graph.add_edges(n, edges); //.to_canonical();

            self.counts.search_tree += 1;
            let search_res = search_tree(new_graph);

            //dbg!(&search_res.orbits);
            
            /*
            let orbits = compute_orbits(n, n, &search_res.automorphisms);
            dbg!(orbits);

            if orbits[search_res.canonical_relabeling[n] as usize] != orbits[n] {
                continue
            }
            */
            //dbg!(search_res.orbits);

            //if orbits[usize::from(search_res.canonical_relabeling[n])] != orbits[n] {
            //if search_res.orbits[usize::from(search_res.canonical_relabeling[n])] != search_res.orbits[n] {
            if usize::from(search_res.canonical_relabeling[n]) != n {
            //if false {
                let cut_vertex = search_res.canonical_relabeling.iter()
                    .enumerate()
                    .filter(|(_, i)| usize::from(**i) == n)
                    .next().unwrap().0;
                    //usize::from(search_res.canonical_relabeling[n]);

                let mut nodes = new_graph.nodes();
                nodes.clear(cut_vertex);

                let mut m_z = new_graph.subgraph(nodes);

                //dbg!(n, cut_vertex, search_res.canonical_relabeling, nodes, m_z, graph);

                let mut found_perm = false;

                self.counts.compute_orbit += 1;
                for perm in check_auto(n, cut_vertex, &search_res.automorphisms) {
                    let mut m_z = m_z;
                    m_z.shuffle2(&perm);
                    //dbg!(n, cut_vertex, perm, m_z);
                    if m_z == graph {
                        found_perm = true;
                        break
                    }
                }

                if !found_perm {
                    continue
                }
            }

            if curr_graphs.contains(&search_res.canonical_graph) {
                //continue
            }
            curr_graphs.insert(search_res.canonical_graph);

            if (self.prune)(&new_graph) {
                continue
            }

            //assert_eq!(graph, graph.to_canonical(), "original: {:?}", orig_graph);
            
            if n + 1 == self.maxn {
                self.graphs.insert(new_graph);
            } else {
                self.enumerate_inner(search_res.automorphisms, new_graph, n+1);
            }
        }
    }

    pub fn enumerate(&mut self) {
        let graph = Graph16::new(1);
        self.enumerate_inner(HashSet::new(), graph, 1);
    }
}
