use crate::bitset::*;
use crate::seq::*;
use crate::iso;

pub struct EdgeIter {
    g: Graph16,
    u: usize,
    v: usize
}

impl Iterator for EdgeIter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        while self.u < 15 {
            while self.v < 16 {
                if self.g.has_edge(self.u, self.v) {
                    self.v += 1;
                    return Some((self.u, self.v-1))
                }
                self.v += 1
            }
            self.u += 1;
            self.v = self.u + 1;
        }
        None
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Graph16 {
    pub g: [u16; 16]
}

impl Graph16 {
    pub const fn new(n: usize) -> Graph16 {
        let mut g = [0; 16];
        // We use a while loop to keep the function const
        let mut i = 0;
        while i < n {
            g[i] = 1 << i;
            i += 1;
        }

        Graph16 {
            g
        }
    }

    pub const fn regular(n: usize) -> Graph16 {
        let mut g = [0; 16];

        let mut i = 0;
        while i < n {
            g[i] = ((1u32 <<n) - 1) as u16;
            i += 1;
        }

        Graph16 {
            g
        }
    }

    pub fn print_graph(&self) {
        for u in self.nodes() {
            for v in self.nodes() {
                if self.has_edge(u, v) {
                    print!("1 ");
                }        
                else {
                    print!("0 ");
                }
            }
            println!("");
        }
    }

    pub fn print_dot(&self) {
        println!("graph {{");
        for u in self.nodes() {
            println!("    {}[shape = point];", u);
        }
        for (u, v) in self.edges() {
            println!("    {} -- {};", u, v);
        }
        println!("}}");
    }

    pub fn add_node(mut self, u: usize) -> Graph16 {
        debug_assert!(u == 0 || self.has_node(u-1));
        debug_assert!(!self.has_node(u));
        self.g[u] |= 1 << u;
        self
    }

    pub fn del_node(mut self, u: usize) -> Graph16 {
        self.g[u] = 0;
        for i in 0..16 {
            self.g[i] &= !(1 << u);
        }
        // TODO: move nodes to fill empty space
        self
    }

    pub fn has_node(&self, u: usize) -> bool {
        self.has_edge(u, u)
    }

    pub const fn add_edge(mut self, u: usize, v: usize) -> Graph16 {
        // commented to keep the function const
        // debug_assert!(u != v);
        self.g[u] |= 1 << v;
        self.g[v] |= 1 << u;
        self
    }

    pub fn add_edges(mut self, u: usize, edges: Bitset16) -> Graph16 {
        self.g[u] |= edges.to_u16();
        for v in edges {
            self.g[v] |= 1 << u;
        }
        self
    }

    pub fn del_edge(mut self, u: usize, v: usize) -> Graph16 {
        debug_assert!(u != v);
        self.g[u] &= !(1 << v);
        self.g[v] &= !(1 << u);
        self
    }

    pub fn has_edge(&self, u: usize, v: usize) -> bool {
        (self.g[u] & (1<<v)) > 0
    }

    pub fn contract_edge(mut self, u: usize, v: usize) -> Graph16 {
        self.g[u] |= self.g[v];
        for i in 0..16 {
            self.g[i] |= ((self.g[i] & (1 << v)) >> v) << u;
        }
        self.del_node(v)
    }

    pub fn swap_nodes(mut self, u: usize, v: usize) -> Graph16 {
        self.g.swap(u, v);
        for i in 0..16 {
            let ue = (self.g[i] & (1 << u)) >> u;
            let ve = (self.g[i] & (1 << v)) >> v;
            self.g[i] = (self.g[i] & !((1 << u) | (1 << v))) | (ue << v) | (ve << u);
        }
        self
    }

    pub fn shuffle1(mut self, permutation: &Seq16) -> Graph16 {
        let mut visited = Bitset16::new();

        for i in 0..permutation.len() {
            if visited.get(i) {
                continue;
            }
            visited.set(i);
            let mut prev = i;
            let mut j = permutation[i] as usize;
            while j != i {
                visited.set(j);
                self = self.swap_nodes(prev, j);
                prev = j;
                j = permutation[j] as usize;
            }
        }
        self
    }

    pub fn shuffle2(mut self, permutation: &Seq16) -> Graph16 {
        let old = self.g;

        for (i, j) in permutation.iter().enumerate() {
            let mut bitset = Bitset16::from_u16(old[i]);
            bitset.shuffle(permutation);
            self.g[*j as usize] = bitset.to_u16();
        }

        self
    }

    pub fn nodes(&self) -> Bitset16 {
        let mut bitset = 0; 
        for i in 0..16 {
            bitset |= self.g[i] & (1 << i);
        }
        Bitset16::from_u16(bitset)
    }

    pub fn edges(&self) -> EdgeIter {
        EdgeIter {
            g: *self,
            u: 0,
            v: 1
        }
    }

    pub fn siblings(&self, u: usize) -> Bitset16 {
        Bitset16::from_u16(self.g[u])
    }

    pub fn to_canonical(self) -> Graph16 {
        iso::SearchTree::new(self).find_canonical()
    }

    pub fn is_canonical(&self) -> bool {
        self == &self.to_canonical()
    }

    pub fn dfs<PreF: FnMut(&Graph16, usize), PostF: FnMut(&Graph16, usize), EdgeF: FnMut(&Graph16, usize, usize, bool)>(&self, mut pre_f: PreF, mut post_f: PostF, mut edge_f: EdgeF) {
        fn traverse<PreF: FnMut(&Graph16, usize), PostF: FnMut(&Graph16, usize), EdgeF: FnMut(&Graph16, usize, usize, bool)>(graph: &Graph16, pre_f: &mut PreF, post_f: &mut PostF, edge_f: &mut EdgeF, visited: &mut Bitset16, u: usize) {
            if visited.get(u) {
                return
            }
            visited.set(u);
            pre_f(graph, u);
            for v in graph.siblings(u) {
                edge_f(graph, u, v, visited.get(v));
                traverse(graph, pre_f, post_f, edge_f, visited, v);
            }
            post_f(graph, u);
        }

        let mut visited = Bitset16::new();

        for i in 0..16 {
            traverse(self, &mut pre_f, &mut post_f, &mut edge_f, &mut visited, i);
        }
    }

    pub fn minors(&self) -> impl Iterator<Item = Graph16> {
        let graph = *self;
        self.nodes().into_iter().map(move |u| graph.del_node(u))
            .chain(self.edges().map(move |(u, v)| graph.del_edge(u, v)))
            .chain(self.edges().map(move |(u, v)| graph.contract_edge(u, v)))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Coloring16 {
    colors: [u8; 16],
}

impl Coloring16 {
    pub fn new() -> Coloring16 {
        Coloring16 {
            colors: [0; 16],
        }
    }

    pub fn defined(&self, u: usize) -> bool {
        self.colors[u] != 0
    }

    pub fn set(&mut self, u: usize, c: u8) {
        debug_assert!(c < 16);
        self.colors[u] = c+1;
    }

    pub fn get(&self, u: usize) -> u8 {
        /*
        if self.colors[u] == 0 {
            // TODO: something better?
            panic!("Trying to get undefined coloring");
        }
        */
        self.colors[u] - 1
    }

    pub fn next_color(&self) -> u8 {
        *self.colors.iter().max().unwrap_or(&0)
    }

    pub fn cells(&self) -> Bitset16 {
        let mut set = Bitset16::new();

        for c in &self.colors {
            if c > &0 {
                set.set((c-1) as usize);
            }
        }

        set
    }

    pub fn get_cell(&self, cell: u8) -> Bitset16 {
        let mut set = Bitset16::new();
        for (i, c) in self.colors.iter().enumerate() {
            if *c == cell+1 {
                set.set(i);
            }
        }
        set
    }

    pub fn discrete(&self) -> bool {
        let mut set = Bitset16::new();

        for c in &self.colors {
            if c > &0 && set.get((c-1) as usize) {
                return false
            } else if c > &0 {
                set.set((c-1) as usize);
            }
        }

        true
    }

    pub fn permutation(&self) -> Option<Seq16> {
        if !self.discrete() {
            return None
        }

        // FIXME: assumes coloring is consecutive

        let mut end = 15;
        let mut perm = Seq16::new();

        for c in &self.colors {
            if *c == 0 {
                perm.push(end);
                end -= 1;
            } else {
                perm.push((c - 1) as usize);
            }
        }
        
        Some(perm)
    }
}
