use crate::seq::{Seq, SmallSeq, SeqPermutations};
use crate::bitset::{self, Intset, Bitset};
use crate::graph::{Graph, BitsetGraph};

pub type RotationSystem16 = SmallRotationSystem<bitset::Bitset16, 16>;
pub type RotationSystem32 = SmallRotationSystem<bitset::Bitset32, 32>;
pub type RotationSystem64 = SmallRotationSystem<bitset::Bitset64, 64>;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Face {
    u0: usize,
    v0: usize,
}

pub trait RotationSystem<G: Graph>: Sized + Clone {
    type EnumIter:  Iterator<Item = Self>;
    type FacesIter: FacesIter<G, Self>;

    fn empty() -> Self;

    fn simple(graph: &G) -> Self;

    fn enumerate(graph: &G) -> Self::EnumIter;

    fn to_graph(&self) -> G;

    fn genus(&self) -> usize;

    fn faces<'a>(&'a self) -> Faces<'a, G, Self>;

    fn face<'a>(&'a self, face: Face) -> FaceIter<'a, G, Self> {
        FaceIter {
            embedding: self,
            face,
            u: face.u0,
            v: face.v0,
            finished: false,
            _marker: std::marker::PhantomData,
        }
    }

    fn face_nodes(&self, face: Face) -> G::Set {
        let mut nodes = G::Set::new();
        for (u, v) in self.face(face) {
            nodes.set(u);
            nodes.set(v);
        }
        nodes
    }

    fn after(&self, u: usize, v: usize) -> usize;

    fn before(&self, u: usize, v: usize) -> usize;

    fn remove_node(&mut self, node: usize);

    fn insert_edge(&mut self, node: usize, after: usize, dest: usize);

    fn insert_edge_any(&mut self, node: usize, dest: usize);

    fn remove_edge_dir(&mut self, u: usize, v: usize);

    fn remove_edge(&mut self, u: usize, v: usize) {
        self.remove_edge_dir(u, v);
        self.remove_edge_dir(v, u);
    }

    fn embed_free_edge(&mut self, u: usize, v: usize) {
        self.insert_edge_any(u, v);
        self.insert_edge_any(v, u);
    }

    fn embed_edge_after(&mut self, u: usize, u_after: usize, v: usize, v_after: usize) {
        self.insert_edge(u, u_after, v);
        self.insert_edge(v, v_after, u);
    }

    fn embed_edge_before(&mut self, u: usize, u_before: usize, v: usize) {
        self.insert_edge(u, self.before(u, u_before), v);
        self.insert_edge_any(v, u);
    }

    fn embed_bisecting_path(&mut self, face: Face, path: &G::Path) -> [Face; 2] {
        let start = path.get(0);
        let end   = path.get(path.len()-1);

        let mut start_u = None;
        let mut end_u = None;

        for (u, v) in self.face(face) {
            if start_u.is_none() && u == start {
                start_u = Some(v);
            }
            if end_u.is_none() && v == end {
                end_u = Some(u);
            }
        }

        self.embed_bisecting_path_after(path, start_u.unwrap(), end_u.unwrap())
    }

    fn embed_bisecting_path_after(&mut self, path: &G::Path, start_u: usize, end_u: usize) -> [Face; 2] {
        let start     = path.get(0);
        let start_snd = path.get(1);
        let end       = path.get(path.len()-1);
        let end_snd   = path.get(path.len()-2);

        self.insert_edge(start, self.before(start, start_u), start_snd);
        self.insert_edge(end, end_u, end_snd);
        if start_snd != end {
            self.insert_edge_any(start_snd, start);
            self.insert_edge_any(end_snd, end);
        }

        if path.len() > 2 {
            for (u, v) in path.iter().skip(1).take(path.len()-3).zip(path.iter().skip(2)) {
                self.embed_free_edge(u, v);
            }
        }

        // This should in theory give the two faces on each side of the bisecting path
        [Face { u0: start, v0: start_snd }, Face { u0: start_snd, v0: start }]
    }

    fn embed_disconnected(&mut self, other: &Self);
}

pub struct FaceIter<'a, G: Graph, R: RotationSystem<G>> {
    embedding: &'a R,
    face: Face,
    u: usize,
    v: usize,
    finished: bool,
    _marker: std::marker::PhantomData<G>,
}

impl<'a, G: Graph, R: RotationSystem<G>> Iterator for FaceIter<'a, G, R> {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<(usize, usize)> {
        if self.finished {
            return None
        }

        let next_v = self.embedding.after(self.v, self.u);

        let old_u = self.u;
        self.u = self.v;
        self.v = next_v;

        if self.u == self.face.u0 && self.v == self.face.v0 {
            self.finished = true;
        }
        Some((old_u, self.u))
    }
}

pub struct Faces<'a, G: Graph, R: RotationSystem<G>> {
    embedding: &'a R,
    iter: R::FacesIter,
}

pub trait FacesIter<G: Graph, R: RotationSystem<G>> {
    fn next_face(&mut self, embedding: &R) -> Option<Face>;
}

impl<'a, G: Graph, R: RotationSystem<G>> Iterator for Faces<'a, G, R> {
    type Item = Face;

    #[inline]
    fn next(&mut self) -> Option<Face> {
        self.iter.next_face(self.embedding)
    }
}

#[derive(Copy, Clone)]
pub struct SmallRotationSystem<B, const N: usize> {
    nodes: B,
    edges: [B; N],
    order: [[u8; N]; N],
    order_inv: [[u8; N]; N],
}

impl<B: Bitset, const N: usize> SmallRotationSystem<B, N> {
    pub fn siblings<'a>(&'a self, u: usize) -> impl 'a + Iterator<Item = usize> {
        let mut v = self.edges[u].smallest();
        let v0 = v;

        std::iter::from_fn(move || {
            if let Some(w) = v {
                let next = usize::from(self.order[u][w]);

                v = Some(next);
                if v == v0 {
                    v = None
                }
                Some(w)
            } else {
                None
            }
        })
    }
}


impl<B: Bitset + Copy, const N: usize> RotationSystem<BitsetGraph<B, N>> for SmallRotationSystem<B, N> {
    type EnumIter = SmallRotationSystemEnumerate<B, N>;
    type FacesIter = SmallFacesIter<B, N>;

    fn empty() -> Self {
        Self {
            nodes: B::new(),
            edges: [B::new(); N],
            order: [[0; N]; N],
            order_inv: [[0; N]; N],
        }
    }

    fn simple(graph: &BitsetGraph<B, N>) -> Self {
        let nodes = graph.nodes();
        let mut edges = [B::new(); N];
        let mut order = [[0; N]; N];
        let mut order_inv = [[0; N]; N];

        for u in graph.nodes().iter() {
            edges[u] = graph.siblings(u);
            for (v, w) in graph.siblings(u).iter()
                .zip(graph.siblings(u).iter().cycle().skip(1))
            {
                order[u][v] = w as u8;
                order_inv[u][w] = v as u8;
            }
        }

        Self {
            nodes,
            edges,
            order,
            order_inv,
        }
    }

    fn enumerate(graph: &BitsetGraph<B, N>) -> SmallRotationSystemEnumerate<B, N> {
        let curr = Self::simple(graph);
        let permutations = [SeqPermutations::empty(); N];

        let flip_node = curr.nodes.iter()
            .filter(|i| curr.edges[*i].count() > 2)
            .next();

        let mut enumerate = SmallRotationSystemEnumerate {
            flip_node,
            curr,
            permutations
        };

        if let Some(i) = enumerate.curr.nodes.smallest() {
            enumerate.new_perm(i);
        }

        enumerate
    }

    fn to_graph(&self) -> BitsetGraph<B, N> {
        let mut graph = BitsetGraph::empty();
        for u in self.nodes.iter() {
            graph.add_node(u);
        }
        for u in self.nodes.iter() {
            for v in self.edges[u].iter() {
                graph.add_edge(u, v);
            }
        }
        graph
    }

    fn genus(&self) -> usize {
        let graph = self.to_graph();
        let edge_count = graph.edges().count();
        let component_count = graph.components().count();
        let face_count = std::cmp::max(1, self.faces().count());
        (3 + edge_count + component_count - 1 - self.nodes.count() - face_count) / 2
    }

    fn faces<'a>(&'a self) -> Faces<'a, BitsetGraph<B, N>, Self> {
        Faces {
            embedding: self,
            iter: SmallFacesIter {
                used: [B::new(); N],
                visited: B::new(),
            },
        }
    }

    #[inline]
    fn after(&self, u: usize, v: usize) -> usize {
        usize::from(self.order[u][v])
    }

    #[inline]
    fn before(&self, u: usize, v: usize) -> usize {
        usize::from(self.order_inv[u][v])
    }

    #[inline]
    fn remove_node(&mut self, u: usize) {
        for v in self.edges[u].iter() {
            self.remove_edge(u, v);
        }
        self.nodes.clear(u);
    }

    #[inline]
    fn insert_edge(&mut self, node: usize, after: usize, dest: usize) {
        self.nodes.set(dest);
        self.edges[node].set(dest);
        let k = self.order[node][after];
        self.order[node][dest] = k;
        self.order_inv[node][dest] = after as u8;
        self.order_inv[node][usize::from(k)] = dest as u8;
        self.order[node][after] = dest as u8;
    }

    #[inline]
    fn insert_edge_any(&mut self, node: usize, dest: usize) {
        if let Some(i) = self.edges[node].smallest() {
            self.insert_edge(node, i, dest);
        } else {
            self.insert_edge(node, dest, dest);
        }
    }

    #[inline]
    fn remove_edge_dir(&mut self, u: usize, v: usize) {
        self.edges[u].clear(v);
        if self.edges[u].is_empty() {
            self.nodes.clear(u);
        }
        let before = usize::from(self.order_inv[u][v]);
        let after  = usize::from(self.order[u][v]);
        self.order[u][before] = after as u8;
        self.order_inv[u][after]  = before as u8;
    }

    fn embed_disconnected(&mut self, other: &Self) {
        debug_assert!(self.nodes.intersection(&other.nodes).is_empty());
        self.nodes = self.nodes.union(&other.nodes);
        for i in 0..N {
            self.edges[i] = self.edges[i].union(&other.edges[i]);
        }
        for u in other.nodes.iter() {
            self.order[u] = other.order[u];
            self.order_inv[u] = other.order_inv[u];
        }
    }
}

impl<B: Bitset + std::fmt::Debug, const N: usize> std::fmt::Debug for SmallRotationSystem<B, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.nodes.iter().map(|u| {
            (u, self.siblings(u).collect::<Vec<_>>())
        })).finish()
    }
}

pub struct SmallFacesIter<B, const N: usize> {
    used: [B; N],
    visited: B,
}

impl<B: Bitset + Copy, const N: usize> FacesIter<BitsetGraph<B, N>, SmallRotationSystem<B, N>> for SmallFacesIter<B, N> {
    #[inline]
    fn next_face(&mut self, embedding: &SmallRotationSystem<B, N>) -> Option<Face> {
        while let Some(u) = embedding.nodes.intersection(&self.visited.invert()).smallest() {
            if let Some(v) = embedding.edges[u]
                .intersection(&self.used[u].invert())
                .smallest()
            {
                let face = Face { u0: u, v0: v };
                for (ki, kj) in embedding.face(face) {
                    self.used[ki].set(kj);
                }
                return Some(face)
            }
            self.visited.set(u);
        }
        None
    }
}

pub struct SmallRotationSystemEnumerate<B, const N: usize> {
    flip_node: Option<usize>,
    curr: SmallRotationSystem<B, N>,
    permutations: [SeqPermutations<N>; N],
}

impl<B: Bitset, const N: usize> SmallRotationSystemEnumerate<B, N> {
    #[inline]
    fn new_perm(&mut self, i: usize) {
        let mut seq: SmallSeq<N> = SmallSeq::new();
        for j in self.curr.edges[i].iter().skip(1) {
            seq.push(j);
        }
        self.permutations[i] = seq.permutations();
    }

    #[inline]
    fn next_perm(&mut self, i: usize) -> bool {
        if let Some(new) = self.permutations[i].next() {
            if let Some(j0) = self.curr.edges[i].smallest() {
                let mut last = j0;
                for next in new.iter() {
                    self.curr.order[i][last] = next as u8;
                    self.curr.order_inv[i][next] = last as u8;
                    last = next;
                }
                self.curr.order[i][last] = j0 as u8;
                self.curr.order_inv[i][j0] = last as u8;
            }
            true
        } else {
            false
        }
    }

    #[inline]
    fn flip_perm(&self) -> bool {
        if let Some(i) = self.flip_node {
            let j = self.curr.edges[i].smallest().unwrap();
            self.curr.order[i][j] < self.curr.order_inv[i][j]
        } else {
            true
        }
    }
}

impl<B: Bitset, const N: usize> Iterator for SmallRotationSystemEnumerate<B, N> {
    type Item = SmallRotationSystem<B, N>;

    #[inline]
    fn next(&mut self) -> Option<SmallRotationSystem<B, N>> {
        for i in self.curr.nodes.iter().rev() {
            while self.next_perm(i) {
                for j in self.curr.nodes.intersection(&B::mask_ge(i+1)).iter() {
                    self.new_perm(j);
                    self.next_perm(j);
                }
                if self.flip_perm() {
                    return Some(self.curr.clone())
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph16;

    #[test]
    fn empty_is_planar() {
        let embedding = RotationSystem16::empty();
        assert_eq!(embedding.genus(), 0);
    }

    #[test]
    fn k1_simple_is_planar() {
        let k1 = Graph16::complete(1);
        let embedding = RotationSystem16::simple(&k1);
        assert_eq!(embedding.genus(), 0);
    }

    #[test]
    fn k1_x10_simple_is_planar() {
        let mut graph = Graph16::empty();
        for i in 0..10 {
            graph.add_node(i);
        }
        let embedding = RotationSystem16::simple(&graph);
        assert_eq!(embedding.genus(), 0);
    }

    #[test]
    fn k1_x10_all_is_planar() {
        let mut graph = Graph16::empty();
        for i in 0..10 {
            graph.add_node(i);
        }
        for embedding in RotationSystem16::enumerate(&graph) {
            assert_eq!(embedding.genus(), 0);
        }
    }

    #[test]
    fn k3_simple_is_planar() {
        let k3 = Graph16::complete(3);
        let embedding = RotationSystem16::simple(&k3);
        assert_eq!(embedding.genus(), 0);
    }

    #[test]
    fn k3_x2_simple_is_planar() {
        let k3 = Graph16::complete(3);
        let mut embedding = RotationSystem16::simple(&k3);

        let mut k3 = Graph16::empty();
        for u in 3..6 {
            k3.add_node(u);
        }
        for u in 3..6 {
            for v in 3..u {
                k3.add_edge(u, v);
            }
        }

        embedding.embed_disconnected(&RotationSystem16::simple(&k3));

        assert_eq!(embedding.genus(), 0);
    }

    #[test]
    fn count_toroidal_embeddings_k5() {
        let k5 = Graph16::complete(5);

        let count = RotationSystem16::enumerate(&k5)
            .filter(|embedding| embedding.genus() == 1)
            .count();

        // @myrvold2018large
        assert_eq!(count, 231);
    }

    #[test]
    fn count_toroidal_embeddings_k33() {
        let mut k33 = Graph16::empty();
        for i in 0..6 {
            k33.add_node(i);
        }
        for i in 0..3 {
            for j in 3..6 {
                k33.add_edge(i, j);
            }
        }

        let count = RotationSystem16::enumerate(&k33)
            .filter(|embedding| embedding.genus() == 1)
            .count();

        // @myrvold2018large
        assert_eq!(count, 20);
    }
}
