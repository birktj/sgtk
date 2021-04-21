use std::hash::Hash;
use crate::bitset::{self, Intset, Bitset};
use crate::seq::{self, Seq};
use crate::permutation::{Permutation, SmallPerm};
use crate::embedding::{RotationSystem, SmallRotationSystem};

pub type Graph16 = BitsetGraph<bitset::Bitset16, 16>;
pub type Graph32 = BitsetGraph<bitset::Bitset32, 32>;
pub type Graph64 = BitsetGraph<bitset::Bitset64, 64>;
pub type Coloring16 = SmallColoring<bitset::Bitset16, 16>;
pub type Coloring32 = SmallColoring<bitset::Bitset32, 32>;
pub type Coloring64 = SmallColoring<bitset::Bitset64, 64>;

pub fn subgraphs<'a, G: Graph + Clone>(graph: &'a G) -> impl 'a + Iterator<Item = G> {
    graph.nodes().iter().map(move |u| {
        let mut graph = graph.clone();
        graph.del_node(u);
        graph
    }).chain(graph.edges().map(move |(u, v)| {
        let mut graph = graph.clone();
        graph.del_edge(u, v);
        graph
    }))
}

pub fn minors<'a, G: Graph + Clone>(graph: &'a G) -> impl 'a + Iterator<Item = G> {
    graph.nodes().iter().map(move |u| {
        let mut graph = graph.clone();
        graph.del_node(u);
        graph
    }).chain(graph.edges().map(move |(u, v)| {
        let mut graph = graph.clone();
        graph.del_edge(u, v);
        graph
    })).chain(graph.edges().map(move |(u, v)| {
        let mut graph = graph.clone();
        graph.contract_edge(u, v);
        graph
    }))
}

pub trait Graph: Sized + Clone {
    const MAXN: usize;
    type Perm: Permutation;
    type Set: Bitset<Perm = Self::Perm>;
    type Path: Seq;
    type Coloring: Coloring<Set = Self::Set, Perm = Self::Perm>;
    type Embedding: RotationSystem<Self>;

    fn empty() -> Self;

    fn complete(n: usize) -> Self where Self: Sized {
        let mut graph = Self::empty();
        for i in 0..n {
            graph.add_node(i);
            for j in 0..i {
                graph.add_edge(i, j);
            }
        }
        graph
    }

    fn add_node(&mut self, u: usize);

    fn del_node(&mut self, u: usize);

    fn del_nodes(&mut self, nodes: &Self::Set);

    fn has_node(&self, u: usize) -> bool;

    fn add_edge(&mut self, u: usize, v: usize);

    fn add_edges(&mut self, u: usize, edges: &Self::Set);

    fn del_edge(&mut self, u: usize, v: usize);

    fn del_edges(&mut self, u: usize, edges: &Self::Set);

    fn has_edge(&self, u: usize, v: usize) -> bool;

    fn siblings(&self, u: usize) -> Self::Set;

    fn contract_edge(&mut self, u: usize, v: usize) {
        self.add_edges(u, &self.siblings(v));
        self.del_node(v);
    }

    fn merge_nodes(&mut self, nodes: &Self::Set) {
        let mut edges = Self::Set::new();
        for i in nodes.iter() {
            edges = edges.union(&self.siblings(i));
        }
        self.del_nodes(nodes);
        if let Some(u) = nodes.smallest() {
            self.add_node(u);
            self.add_edges(u, &edges);
        }
    }

    fn swap_nodes(&mut self, u: usize, v: usize) {
        let u_edges = self.siblings(u);
        let v_edges = self.siblings(v);
        self.del_node(u);
        self.del_node(v);
        self.add_node(u);
        self.add_node(v);
        self.add_edges(u, &v_edges);
        self.add_edges(v, &u_edges);

        /*
        self.g.swap(u, v);
        for i in 0..16 {
            let ue = (self.g[i] & (1 << u)) >> u;
            let ve = (self.g[i] & (1 << v)) >> v;
            self.g[i] = (self.g[i] & !((1 << u) | (1 << v))) | (ue << v) | (ve << u);
        }
        */
    }

    fn trim(&mut self) {
        let mut i = 0;
        for u in self.nodes().iter() {
            if u <= i {
                i += 1;
                continue
            }
            while self.has_node(i) {
                i += 1;
            }
            let u_edges = self.siblings(u);
            self.del_node(u);
            self.add_node(i);
            self.add_edges(i, &u_edges);
            i += 1;
        }
    }

    fn convert<G: Graph>(&self) -> G {
        let mut graph = G::empty();
        for u in self.nodes().iter() {
            graph.add_node(u);
        }
        for (u, v) in self.edges() {
            graph.add_edge(u, v);
        }
        graph
    }

    fn shuffle(&mut self, permutation: &Self::Perm);

    fn nodes(&self) -> Self::Set;

    fn edges_from_to(&self, from: Self::Set, to: Self::Set) -> EdgeIter<Self> {
        let iter_u = from.iter();
        EdgeIter {
            g: self,
            u: 0,
            to,
            from ,
            iter_u,
            iter_v: Self::Set::new().iter(),
        }
    }

    fn edges(&self) -> EdgeIter<Self> {
        self.edges_from_to(self.nodes(), self.nodes())
    }

    fn edges_from(&self, from: Self::Set) -> EdgeIter<Self> {
        self.edges_from_to(from, self.nodes())
    }

    fn subgraph(&self, selected: &Self::Set) -> Self;

    fn is_supergraph(&self, other: &Self) -> bool;

    fn union(&mut self, other: &Self);

    fn difference(&mut self, other: &Self);

    fn neighbouring(&self, nodes: &Self::Set) -> Self {
        let mut selection = Self::Set::new();
        for i in nodes.iter() {
            selection.set(i);
            selection = selection.union(&self.siblings(i));
        }
        self.subgraph(&selection)
    }

    fn bipartite_split(&self, a: &Self::Set, b: &Self::Set) -> Self;

    fn is_connected(&self) -> bool {
        let nodes = self.nodes();

        if let Some(i) = nodes.smallest() {
            self.get_component(i) == nodes
        } else {
            true
        }
    }

    fn get_component(&self, u: usize) -> Self::Set {
        let mut visited = Self::Set::new();
        let mut queue   = Self::Set::new();
        queue.set(u);

        while let Some(i) = queue.intersection(&visited.invert()).smallest() {
            visited.set(i);
            queue = queue.union(&self.siblings(i));
        }

        visited
    }

    fn to_canonical(self) -> Self
        where Self: Ord, Self::Perm: Eq + Hash, Self::Path: Eq + Hash
    {
        crate::iso::search_tree(self).canonical_graph
    }

    fn is_canonical(&self) -> bool 
        where Self: Eq + Ord, Self::Perm: Eq + Hash, Self::Path: Eq + Hash
    {
        self == &self.clone().to_canonical()
    }

    fn is_planar(&self) -> bool {
        if self.is_connected() {
            crate::planar::fastdmp(self).is_some()
        } else {
            for component in self.clone().components() {
                if crate::planar::fastdmp(&component).is_none() {
                    return false
                }
            }
            true
        }
    }

    fn spanning_tree(&self) -> Self {
        struct Dfs<'a, G: Graph> {
            graph: &'a G,
            res: G,
            visited: G::Set,
        }

        impl<'a, G: Graph> Dfs<'a, G> {
            fn dfs(&mut self, u: usize) {
                self.visited.set(u);

                for v in self.graph.siblings(u).iter() {
                    if !self.visited.get(v) {
                        self.res.add_node(v);
                        self.res.add_edge(u, v);
                        self.dfs(v);
                    }
                }
            }
        }

        let mut dfs = Dfs {
            graph: self,
            res: Self::empty(),
            visited: Self::Set::new(),
        };

        while let Some(u) = dfs.visited.invert().smallest() {
            dfs.res.add_node(u);
            dfs.dfs(u);
        }

        dfs.res
    }

    fn cycle(&self) -> Option<Self> where Self: Clone {
        struct Dfs<G: Graph> {
            graph: G,
            curr: G,
            visited: G::Set,
            found: bool,
        }

        impl<G: Graph> Dfs<G> {
            fn dfs(&mut self, u: usize) -> Option<usize> {
                if self.visited.get(u) {
                    self.curr.add_node(u);
                    self.found = true;
                    return Some(u)
                }
                self.visited.set(u);

                for v in self.graph.siblings(u).iter() {
                    self.graph.del_edge(u, v);
                    if let Some(w) = self.dfs(v) {
                        self.curr.add_node(u);
                        self.curr.add_edge(u, v);
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
            graph: self.clone(),
            curr: Self::empty(),
            visited: Self::Set::new(),
            found: false,
        };

        let nodes = self.nodes();
        while let Some(u) = dfs.visited.invert().intersection(&nodes).smallest() {
            dfs.dfs(u);
            if dfs.found {
                return Some(dfs.curr)
            }
        }

        None
    }

    fn from_path(path: &Self::Path) -> Self where Self: Sized {
        let mut graph = Self::empty();

        for (u, v) in path.iter().zip(path.iter().skip(1)) {
            graph.add_node(u);
            graph.add_node(v);
            graph.add_edge(u, v);
        }

        graph
    }

    fn path(&self, start: usize, goal: &Self::Set) -> Option<Self::Path> {
        struct Dfs<'a, G: Graph> {
            graph: &'a G,
            goal: &'a G::Set,
            visited: G::Set,
            path: G::Path,
        }

        impl<'a, G: Graph> Dfs<'a, G> {
            fn dfs(&mut self, u: usize) -> bool {
                if self.visited.get(u) {
                    return false
                }
                if self.goal.get(u) {
                    self.path.push(u);
                    return true
                }
                self.visited.set(u);

                for v in self.graph.siblings(u).iter() {
                    if self.dfs(v) {
                        self.path.push(u);
                        return true
                    }
                }
                false
            }
        }

        let mut dfs = Dfs {
            graph: self,
            goal,
            visited: Self::Set::new(),
            path: Self::Path::new(),
        };

        if dfs.dfs(start) {
            dfs.path.reverse();
            Some(dfs.path)
        } else {
            None
        }
    }

    fn components(self) -> ComponentIter<Self> {
        let visited = self.nodes().invert();
        ComponentIter {
            graph: self,
            visited,
        }
    }

    /*
    fn subgraphs<'a>(&'a self) -> impl 'a + Iterator<Item = Self> where Self: Clone {
        self.nodes().into_iter().map(move |u| {
            let mut graph = self.clone();
            graph.del_node(u);
            graph
        }).chain(self.edges().map(move |(u, v)| {
            let mut graph = self.clone();
            graph.del_edge(u, v);
            graph
        }))
    }

    fn minors<'a>(&'a self) -> impl 'a + Iterator<Item = Graph16> {
        self.nodes().into_iter().map(move |u| {
            let mut graph = self.clone();
            graph.del_node(u);
            graph
        }).chain(self.edges().map(move |(u, v)| {
            let mut graph = self.clone();
            graph.del_edge(u, v);
            graph
        })).chain(self.edges().map(move |(u, v)| {
            let mut graph = self.clone();
            graph.contract_edge(u, v);
            graph
        }))
    }
    */
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BitsetGraph<B, const N: usize> {
    g: [B; N],
}

impl<B: Bitset + Copy, const N: usize> BitsetGraph<B, N> {
    pub fn from_raw(raw: &[B]) -> Self {
        let mut graph = Self::empty();
        for (i, u) in raw.iter().enumerate() {
            graph.g[i] = *u;
        }
        graph
    }

    pub fn to_raw(self) -> [B; N] {
        self.g
    }
}

impl<B: Bitset + Copy, const N: usize> Graph for BitsetGraph<B, N> {
    const MAXN: usize = N;
    type Perm = B::Perm;
    type Set = B;
    type Path = seq::SmallSeq<N>;
    type Coloring = SmallColoring<B, N>;
    type Embedding = SmallRotationSystem<B, N>;

    fn empty() -> Self {
        Self {
            g: [B::new(); N],
        }
    }

    #[inline]
    fn add_node(&mut self, u: usize) {
        self.g[u].set(u);
    }

    #[inline]
    fn del_node(&mut self, u: usize) {
        self.g[u] = B::new();
        for i in 0..N {
            self.g[i].clear(u);
        }
    }

    #[inline]
    fn del_nodes(&mut self, nodes: &Self::Set) {
        for u in nodes.iter() {
            self.g[u] = B::new();
        }
        let inv = nodes.invert();
        for i in 0..N {
            self.g[i] = self.g[i].intersection(&inv);
        }
    }

    #[inline]
    fn has_node(&self, u: usize) -> bool {
        self.g[u].get(u)
    }

    #[inline]
    fn nodes(&self) -> Self::Set {
        let mut res = B::new();
        for i in 0..N {
            res.set_val(i, self.has_node(i));
        }
        res
    }

    #[inline]
    fn add_edge(&mut self, u: usize, v: usize) {
        debug_assert!(self.has_node(u));
        debug_assert!(self.has_node(v));
        self.g[u].set(v);
        self.g[v].set(u);
    }

    #[inline]
    fn add_edges(&mut self, u: usize, edges: &Self::Set) {
        self.g[u] = self.g[u].union(edges);
        for v in edges.iter() {
            self.g[v].set(u);
        }
    }

    #[inline]
    fn del_edge(&mut self, u: usize, v: usize) {
        self.g[u].clear(v);
        self.g[v].clear(u);
    }

    #[inline]
    fn del_edges(&mut self, u: usize, edges: &Self::Set) {
        self.g[u] = self.g[u].intersection(&edges.invert());
        for v in edges.iter() {
            self.g[v].clear(u);
        }
    }

    #[inline]
    fn has_edge(&self, u: usize, v: usize) -> bool {
        self.g[u].get(v)
    }

    #[inline]
    fn siblings(&self, u: usize) -> Self::Set {
        let mut res = self.g[u];
        res.clear(u);
        res
    }

    #[inline]
    fn shuffle(&mut self, permutation: &Self::Perm) {
        let old = self.g;

        for (i, j) in permutation.iter() {
            let mut bitset = old[i];
            bitset.shuffle(permutation);
            self.g[j] = bitset;
        }
    }


    #[inline]
    fn subgraph(&self, selected: &Self::Set) -> Self {
        let mut new = Self::empty();
        for i in selected.iter() {
            new.g[i] = self.g[i].intersection(selected);
        }
        new
    }

    #[inline]
    fn is_supergraph(&self, other: &Self) -> bool {
        let mut res = true;
        for i in 0..N {
            res = res && self.g[i].is_superset(&other.g[i]);
        }
        res
    }

    #[inline]
    fn union(&mut self, other: &Self) {
        for i in 0..N {
            self.g[i] = self.g[i].union(&other.g[i]);
        }
    }

    #[inline]
    fn difference(&mut self, other: &Self) {
        for i in 0..N {
            self.g[i] = self.g[i].difference(&other.g[i]);
            self.g[i].set_val(i, !(self.g[i].get(i) && other.g[i].get(i)));
        }
    }

    #[inline]
    fn bipartite_split(&self, a: &Self::Set, b: &Self::Set) -> Self {
        let mut new = Self::empty();
        for i in a.iter() {
            new.g[i] = self.g[i].intersection(b);
            new.g[i].set_val(i, self.g[i].get(i));
        }
        for i in b.iter() {
            new.g[i] = self.g[i].intersection(a);
            new.g[i].set_val(i, self.g[i].get(i));
        }
        new
    }
}

impl<B: Bitset<Perm = SmallPerm<N>> + Copy, const N: usize> std::fmt::Debug for BitsetGraph<B, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.nodes().iter().map(|u| {
            (u, self.siblings(u).iter().collect::<Vec<_>>())
        })).finish()
    }
}

pub struct EdgeIter<'a, G: Graph> {
    g: &'a G,
    u: usize,
    to: G::Set,
    from: G::Set,
    iter_u: <G::Set as Bitset>::Iter,
    iter_v: <G::Set as Bitset>::Iter,
}

impl<'a, G: Graph> Iterator for EdgeIter<'a, G> {
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
            let mask = self.to.intersection(&self.from.invert().union(&G::Set::mask_le(self.u)));
            self.iter_v = self.g.siblings(self.u).intersection(&mask).iter();
        }
    }
}

pub struct ComponentIter<G: Graph> {
    graph: G,
    visited: G::Set,
}

impl<G: Graph> Iterator for ComponentIter<G> {
    type Item = G;

    #[inline]
    fn next(&mut self) -> Option<G> {
        let i = self.visited.invert().smallest()?;
        let component = self.graph.get_component(i);
        self.visited = self.visited.union(&component);
        Some(self.graph.subgraph(&component))
    }
}


pub trait Coloring: Clone {
    type Perm: Permutation;
    type Set: Bitset<Perm = Self::Perm>;

    fn new() -> Self;

    fn defined(&self, u: usize) -> bool;

    fn set(&mut self, u: usize, c: usize);

    fn get(&self, u: usize) -> usize;

    fn next_color(&self) -> usize;

    fn cells(&self) -> Self::Set;

    fn get_cell(&self, cell: usize) -> Self::Set;

    fn discrete(&self) -> bool;

    fn individualize(&mut self, u: usize);

    fn permutation(&self) -> Option<Self::Perm>;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SmallColoring<S, const N: usize> {
    colors: [u8; N],
    _marker: std::marker::PhantomData<S>,
}

impl<S, const N: usize> std::fmt::Debug for SmallColoring<S, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.colors)
    }
}

impl<S: Bitset, const N: usize> Coloring for SmallColoring<S, N> {
    type Set = S;
    type Perm = S::Perm;

    fn new() -> Self {
        Self {
            colors: [0; N],
            _marker: std::marker::PhantomData,
        }
    }

    fn defined(&self, u: usize) -> bool {
        self.colors[u] != 0
    }

    fn set(&mut self, u: usize, c: usize) {
        debug_assert!(c < N);
        self.colors[u] = c as u8 + 1;
    }

    fn get(&self, u: usize) -> usize {
        /*
        if self.colors[u] == 0 {
            // TODO: something better?
            panic!("Trying to get undefined coloring");
        }
        */
        usize::from(self.colors[u] - 1)
    }

    fn next_color(&self) -> usize {
        usize::from(*self.colors.iter().max().unwrap_or(&0))
    }

    fn cells(&self) -> S {
        let mut set = S::new();

        for c in &self.colors {
            if c > &0 {
                set.set((c-1) as usize);
            }
        }

        set
    }

    fn get_cell(&self, cell: usize) -> S {
        let mut set = S::new();
        for (i, c) in self.colors.iter().enumerate() {
            if usize::from(*c) == cell+1 {
                set.set(i);
            }
        }
        set
    }

    fn discrete(&self) -> bool {
        let mut set = S::new();

        for c in &self.colors {
            if c > &0 && set.get((c-1) as usize) {
                return false
            } else if c > &0 {
                set.set((c-1) as usize);
            }
        }

        true
    }

    fn individualize(&mut self, u: usize) {
        let cu = self.colors[u];
        for v in 0..N {
            self.colors[v] = if u != v && self.colors[v] >= cu {
                self.colors[v] + 1
            } else {
                self.colors[v]
            };
        }
    }

    fn permutation(&self) -> Option<Self::Perm> {
        // FIXME: assumes coloring is consecutive

        let mut end = N;
        Self::Perm::from_iter(self.colors.iter().map(|c| {
            if *c == 0 {
                end -= 1;
                end
            } else {
                usize::from(*c - 1)
            }
        }).enumerate())
    }
}
