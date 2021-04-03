use crate::bitset::*;
use crate::seq::*;
use crate::iso;

pub struct EdgeIter16 {
    g: Graph16,
    u: usize,
    to: Bitset16,
    from: Bitset16,
    iter_u: IterBitset16,
    iter_v: IterBitset16,
}

impl Iterator for EdgeIter16 {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<(usize, usize)> {
        loop {
            while let Some(v) = self.iter_v.next() {
                if self.g.has_edge(self.u, v) {
                    return Some((self.u, v))
                }
            }
            self.u = self.iter_u.next()?;
            let mask = self.to.to_u16() & (((1 << self.u) - 1) | !self.from.to_u16());
            self.iter_v = Bitset16::from_u16(self.g.g[self.u] & mask).into_iter();
        }
    }
}

pub struct ComponentIter16 {
    graph: Graph16,
    visited: Bitset16,
}

/*
impl ComponentIter16 {
    fn dfs_component(&mut self, i: usize) {
        if self.visited.get(i) {
            return
        }
        self.visited.set(i);
        self.curr.set(i);

        for j in self.graph.siblings(i) {
            self.dfs_component(j);
        }
    }
}
*/

impl Iterator for ComponentIter16 {
    type Item = Graph16;

    #[inline]
    fn next(&mut self) -> Option<Graph16> {
        let i = self.visited.invert().smallest()?;
        let component = self.graph.get_component(i);
        self.visited = self.visited.union(&component);
        Some(self.graph.subgraph(component))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

    pub fn from_path(path: &Seq16) -> Graph16 {
        let mut graph = Graph16::new(0);
        for (u, v) in path.iter().zip(path.iter().skip(1)) {
            let u = usize::from(*u);
            let v = usize::from(*v);
            graph = graph.add_node(u).add_node(v).add_edge(u, v);
        }
        graph
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
        //debug_assert!(u == 0 || self.has_node(u-1));
        //debug_assert!(!self.has_node(u));
        self.g[u] |= 1 << u;
        self
    }

    fn trim(mut self) -> Graph16 {
        // TODO: can this be done more efficently?
        let mut i = 0;
        let mut j = 0;
        while i < 15 && j < 15 {
            while j < 15 && self.g[j] == 0 {
                j += 1;
            }
            if i != j && j < 15 {
                self = self.swap_nodes(i, j);
            }
            i += 1;
            j += 1;
        }
        self
    }

    #[inline]
    pub fn del_node(mut self, u: usize) -> Graph16 {
        self.g[u] = 0;
        for i in 0..16 {
            self.g[i] &= !(1 << u);
        }
        //self.trim()
        self
    }

    #[inline]
    pub fn has_node(&self, u: usize) -> bool {
        self.has_edge(u, u)
    }

    #[inline]
    pub const fn add_edge(mut self, u: usize, v: usize) -> Graph16 {
        // commented to keep the function const
        // debug_assert!(u != v);
        self.g[u] |= 1 << v;
        self.g[v] |= 1 << u;
        self
    }

    #[inline]
    pub fn add_edges(mut self, u: usize, edges: Bitset16) -> Graph16 {
        self.g[u] |= edges.to_u16();
        for v in edges {
            self.g[v] |= 1 << u;
        }
        self
    }

    #[inline]
    pub fn del_edge(mut self, u: usize, v: usize) -> Graph16 {
        debug_assert!(u != v);
        self.g[u] &= !(1 << v);
        self.g[v] &= !(1 << u);
        self
    }

    #[inline]
    pub fn del_edges(mut self, u: usize, mut edges: Bitset16) -> Graph16 {
        self.g[u] &= !edges.to_u16();
        for v in edges {
            self.g[v] &= !(1 << u);
        }
        self
    }

    #[inline]
    pub fn has_edge(&self, u: usize, v: usize) -> bool {
        (self.g[u] & (1<<v)) > 0
    }

    #[inline]
    pub fn contract_edge(mut self, u: usize, v: usize) -> Graph16 {
        self.g[u] |= self.g[v];
        for i in 0..16 {
            self.g[i] |= ((self.g[i] & (1 << v)) >> v) << u;
        }
        self.del_node(v)
    }

    #[inline]
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

    #[inline]
    pub fn nodes(&self) -> Bitset16 {
        let mut bitset = 0; 
        for i in 0..16 {
            bitset |= self.g[i] & (1 << i);
        }
        Bitset16::from_u16(bitset)
    }

    pub fn edges(&self) -> EdgeIter16 {
        let nodes = self.nodes();
        EdgeIter16 {
            g: *self,
            u: 0,
            to: nodes,
            from: nodes,
            iter_u: nodes.into_iter(),
            iter_v: Bitset16::new().into_iter(),
        }
    }

    pub fn edges_from(&self, from: Bitset16) -> EdgeIter16 {
        let nodes = self.nodes();
        EdgeIter16 {
            g: *self,
            u: 0,
            to: nodes,
            from,
            iter_u: from.into_iter(),
            iter_v: Bitset16::new().into_iter(),
        }
    }

    pub fn edges_from_to(&self, from: Bitset16, to: Bitset16) -> EdgeIter16 {
        EdgeIter16 {
            g: *self,
            u: 0,
            to,
            from,
            iter_u: from.into_iter(),
            iter_v: Bitset16::new().into_iter(),
        }
    }

    #[inline]
    pub fn siblings(&self, u: usize) -> Bitset16 {
        Bitset16::from_u16(self.g[u] & !(1 << u))
    }

    #[inline]
    pub fn subgraph(&self, selected: Bitset16) -> Graph16 {
        let mut new = *self;
        for i in 0..16 {
            new.g[i] = Bitset16::from_u16(self.g[i]).intersection(&selected).to_u16();
        }
        for i in 0..16 { // selected.invert() {
            new.g[i] &= if selected.get(i) { 0xffff } else { 0 };
        }
        /*
        for i in 0..16 {
            new.g[i] &= if selected.invert().get(i) { 0 } else { 0xffff };
        }
        */
        //new.trim()
        new
    }

    #[inline]
    pub fn union(mut self, other: &Graph16) -> Graph16 {
        for i in 0..16 {
            self.g[i] = self.g[i] | other.g[i];
        }
        self
    }

    #[inline]
    pub fn difference(&self, other: &Graph16) -> Graph16 {
        let mut new = *self;
        for i in 0..16 {
            new.g[i] = new.g[i] & !other.g[i];
        }
        for i in 0..16 {
            new.g[i] |= if new.g[i] != 0 { 1 << i } else { 0 };
        }
        /*
        let mask = new.nodes().to_u16();
        for i in 0..16 {
            new.g[i] &= new.g[i] & mask;
        }
        */
        new
    }

    #[inline]
    pub fn neighbouring(&self, nodes: Bitset16) -> Graph16 {
        let mut selection = Bitset16::new();
        for i in nodes {
            selection = selection.union(&Bitset16::from_u16(self.g[i]));
        }
        self.subgraph(selection)
    }

    #[inline]
    pub fn bipartite_split(&self, a: Bitset16, b: Bitset16) -> Graph16 {
        let mut new = Graph16::new(0);
        for i in a {
            new.g[i] = self.g[i] & (b.to_u16() | (1 << i));
        }
        for i in b {
            new.g[i] = self.g[i] & (a.to_u16() | (1 << i));
        }
        new
    }

    pub fn is_connected(&self) -> bool {
        let nodes = self.nodes();

        if let Some(i) = nodes.smallest() {
            self.get_component(i) == nodes
        } else {
            true
        }
    }

    pub fn get_component(&self, u: usize) -> Bitset16 {
        let mut visited = Bitset16::new();
        let mut queue = Bitset16::new();
        queue.set(u);

        while let Some(i) = queue.intersection(&visited.invert()).smallest() {
            visited.set(i);
            queue = queue.union(&self.siblings(i));
        }

        visited
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

    pub fn spanning_tree(&self) -> Graph16 {
        let mut graph = Graph16::new(0);
        self.dfs(|_, _| {}, |_, _| {}, |_, u, v, vis| {
           if !vis {
               graph.g[u] |= 1 << u;
               graph.g[v] |= 1 << v;
               graph = graph.add_edge(u, v);
           } 
        });

        graph.trim()
    }

    pub fn cycle(&self) -> Option<Graph16> {
        struct Dfs {
            graph: Graph16,
            curr: Graph16,
            visited: Bitset16,
            found: bool,
        }

        impl Dfs {
            fn dfs(&mut self, u: usize) -> Option<usize> {
                if self.visited.get(u) {
                    self.curr = self.curr.add_node(u);
                    self.found = true;
                    return Some(u)
                }
                self.visited.set(u);

                for v in self.graph.siblings(u) {
                    self.graph = self.graph.del_edge(u, v);
                    if let Some(w) = self.dfs(v) {
                        self.curr = self.curr.add_node(u).add_edge(u, v);
                        if u != w {
                            return Some(w)
                        }
                    }
                    if self.found {
                        return None
                    }
                }
                None
            }
        }

        let mut dfs = Dfs {
            graph: *self,
            curr: Graph16::new(0),
            visited: Bitset16::new(),
            found: false,
        };

        let nodes = dfs.graph.nodes();
        while let Some(u) = dfs.visited.invert().intersection(&nodes).smallest() {
            dfs.dfs(u);
            if dfs.found {
                return Some(dfs.curr)
            }
        }

        None
    }

    pub fn path(&self, start: usize, goal: Bitset16) -> Option<Seq16> {
        struct Dfs {
            graph: Graph16,
            goal: Bitset16,
            visited: Bitset16,
            path: Seq16,
        }

        impl Dfs {
            fn dfs(&mut self, u: usize) -> bool {
                if self.visited.get(u) {
                    return false
                }
                if self.goal.get(u) {
                    self.path.push(u);
                    return true
                }
                self.visited.set(u);

                for v in self.graph.siblings(u) {
                    if self.dfs(v) {
                        self.path.push(u);
                        return true
                    }
                }
                false
            }
        }

        let mut dfs = Dfs {
            graph: *self,
            goal,
            visited: Bitset16::new(),
            path: Seq16::new(),
        };

        if dfs.dfs(start) {
            dfs.path.reverse();
            Some(dfs.path)
        } else {
            None
        }
    }

    pub fn components(self) -> ComponentIter16 {
        ComponentIter16 {
            graph: self,
            visited: self.nodes().invert(),
        }
    }

    pub fn map_components<F: FnMut(&Graph16)>(&self, mut f: F) {
        let mut p = [0; 16];
        let mut s = [1; 16];
        for i in 0..16 {
            p[i] = i;
        }

        let mut union = |mut i, mut j| {
            while p[i] != i {
                i = p[i];
            }
            while p[j] != j {
                j = p[j];
            }
            if s[i] > s[j] {
                p[j] = i;
            } else {
                p[i] = j;
            }
        };

        for (i, j) in self.edges() {
            union(i, j);
        }

        let find = |mut i| {
            while p[i] != i {
                i = p[i];
            }
            i
        };

        for c in 0..16 {
            let mut map = [0; 16];
            let mut n = 0;
            for i in self.nodes() {
                if find(i) == c {
                    map[i] = n;
                    n += 1;
                }
            }
            let mut graph = Graph16::new(n);
            for (i, j) in self.edges() {
                graph.add_edge(map[i], map[j]);
            }
            f(&graph);
        }
    }

    pub fn minors(&self) -> impl Iterator<Item = Graph16> {
        let graph = *self;
        self.nodes().into_iter().map(move |u| graph.del_node(u))
            .chain(self.edges().map(move |(u, v)| graph.del_edge(u, v)))
            .chain(self.edges().map(move |(u, v)| graph.contract_edge(u, v)))
    }
}

impl std::fmt::Debug for Graph16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.nodes().into_iter().map(|u| {
            (u, self.siblings(u).into_iter().collect::<Vec<_>>())
        })).finish()
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
