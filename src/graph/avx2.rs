use core::arch::x86_64::*;
use std::mem::MaybeUninit;
use super::{SmallColoring, Graph};
use crate::prelude::*;
use crate::bitset::Bitset16;
use crate::seq::SmallSeq;
use crate::permutation::SmallPerm;
use crate::embedding::SmallRotationSystem;

const NODEMASK: Graph16 = {
    let mut g = [0; 16];
    let mut i = 0;
    while i < 16 {
        g[i] = 1 << i;
        i += 1;
    }
    Graph16 {
        g
    }
};

#[inline(always)]
fn u16_to_mask(m: u16) -> __m256i {
    unsafe {
        let nodemask = NODEMASK.to_simd();
        let m = _mm256_set1_epi16(m as i16);
        let m = _mm256_and_si256(m, nodemask);
        _mm256_cmpgt_epi16(m, _mm256_setzero_si256())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(align(32))]
pub struct Graph16 {
    g: [u16; 16],
}

impl Graph16 {
    //#[inline]
    pub fn from_raw(raw: &[u16]) -> Self {
        unsafe {
            assert!(raw.len() == 16);
            let g = _mm256_loadu_si256(raw.as_ptr() as *const __m256i);
            Self::from_simd(g)
        }
    }

    //#[inline]
    pub fn to_raw(self) -> [u16; 16] {
        self.g
    }

    #[inline(always)]
    fn to_simd(&self) -> __m256i {
        unsafe { _mm256_load_si256(self.g.as_ptr() as *const __m256i) }
    }

    #[inline(always)]
    fn from_simd(g: __m256i) -> Self {
        unsafe {
            let mut res: MaybeUninit<Self> = MaybeUninit::uninit();
            _mm256_store_si256(res.as_mut_ptr() as *mut __m256i, g);
            res.assume_init()
        }
    }
}

impl Graph for Graph16 {
    const MAXN: usize = 16;
    type Perm = SmallPerm<16>;
    type Set = Bitset16;
    type Path = SmallSeq<16>;
    type Coloring = SmallColoring<Bitset16, 16>;
    type Embedding = SmallRotationSystem<Bitset16, Self, 16>;

    fn empty() -> Self {
        Self {
            g: [0; 16],
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        unsafe {
            let g = self.to_simd();
            _mm256_testz_si256(g, _mm256_setzero_si256()) != 0
        }
    }

    #[inline]
    fn add_node(&mut self, u: usize) {
        self.g[u] |= 1 << u;
    }

    #[inline]
    fn del_node(&mut self, u: usize) {
        self.g[u] = 0;
        /*
        for i in 0..16 {
            self.g[i] &= !(1 << u);
        }
        */
        // TODO: compiler can do this
        unsafe {
            let g = self.to_simd();
            let m = _mm256_set1_epi16(!(1 << u));
            let r = _mm256_and_si256(g, m);
            *self = Self::from_simd(r);
        }
    }

    #[inline]
    fn del_nodes(&mut self, nodes: &Bitset16) {
        unsafe {
            let mask = u16_to_mask(nodes.invert().to_u16());
            let g = self.to_simd();
            let g = _mm256_and_si256(g, mask);
            let m = _mm256_set1_epi16(nodes.invert().to_u16() as i16);
            let g = _mm256_and_si256(g, m);
            *self = Self::from_simd(g);
        }

        /*
        for u in nodes.iter() {
            self.g[u] = 0;
        }
        let inv = nodes.invert().to_u16();
        for i in 0..16 {
            self.g[i] = self.g[i] & inv;
        }
        */
    }

    #[inline]
    fn has_node(&self, u: usize) -> bool {
        self.g[u] & (1 << u) != 0
    }

    #[inline]
    fn nodes(&self) -> Bitset16 {
        // Autovectorization is slightly unreliable
        unsafe {
            let m = NODEMASK.to_simd();
            let g = self.to_simd();

            let g = _mm256_and_si256(g, m);
            let a = _mm256_extracti128_si256(g, 0);
            let b = _mm256_extracti128_si256(g, 1);

            let a = _mm_or_si128(a, b);
            let b = _mm_shuffle_epi32(a, 78);

            let a = _mm_or_si128(a, b);
            let b = _mm_shuffle_epi32(a, 229);

            let a = _mm_or_si128(a, b);
            let b = _mm_srli_epi32(a, 16);

            let a = _mm_or_si128(a, b);
            let r = _mm_extract_epi16(a, 0);

            Bitset16::from_u16(r as u16)
        }

        /*
        // Compiler manages this on its own
        let mut res = 0;
        for i in 0..16 {
            res |= (self.has_node(i) as u16) << i;
        }
        Bitset16::from_u16(res)
        */
    }

    #[inline]
    fn add_edge(&mut self, u: usize, v: usize) {
        debug_assert!(self.has_node(u));
        debug_assert!(self.has_node(v));
        self.g[u] |= 1 << v;
        self.g[v] |= 1 << u;
    }

    #[inline]
    fn add_edges(&mut self, u: usize, edges: &Bitset16) {
        self.g[u] = self.g[u] | edges.to_u16();
        /*
        for v in edges.iter() {
            self.g[v] |= 1 << u;
        }
        */
        unsafe {
            let mask = u16_to_mask(edges.to_u16());
            let m = _mm256_set1_epi16(1 << u);
            let m = _mm256_and_si256(m, mask);
            let g = self.to_simd();
            let g = _mm256_or_si256(g, m);
            *self = Self::from_simd(g);
        }
    }

    #[inline]
    fn del_edge(&mut self, u: usize, v: usize) {
        self.g[u] &= !(1 << v);
        self.g[v] &= !(1 << u);
    }

    #[inline]
    fn del_edges(&mut self, u: usize, edges: &Bitset16) {
        self.g[u] = self.g[u] & edges.invert().to_u16();
        /*
        for v in edges.iter() {
            self.g[v] &= !(1 << u);
        }
        */
        unsafe {
            let mask = u16_to_mask(edges.to_u16());
            let m = _mm256_set1_epi16(!(1 << u));
            let m = _mm256_and_si256(m, mask);
            let g = self.to_simd();
            let g2 = _mm256_and_si256(g, m);
            let g = _mm256_blendv_epi8(g, g2, mask);
            *self = Self::from_simd(g);
        }
    }

    #[inline]
    fn has_edge(&self, u: usize, v: usize) -> bool {
        self.g[u] & (1 << v) != 0
    }

    #[inline]
    fn siblings(&self, u: usize) -> Self::Set {
        Bitset16::from_u16(self.g[u] & !(1 << u))
    }

    #[inline]
    fn shuffle(&mut self, permutation: &Self::Perm) {
        let old = self.g;

        for (i, j) in permutation.iter() {
            let mut bitset = Bitset16::from_u16(old[i]);
            bitset.shuffle(permutation);
            self.g[j] = bitset.to_u16();
        }
    }


    #[inline]
    fn subgraph(&self, selected: &Self::Set) -> Self {
        unsafe {
            let mask = u16_to_mask(selected.to_u16());
            let g = self.to_simd();
            let g = _mm256_and_si256(g, mask);
            let mask = _mm256_set1_epi16(selected.to_u16() as i16);
            let g = _mm256_and_si256(g, mask);
            Self::from_simd(g)
        }
        /*
        let mut new = Self::empty();
        for i in selected.iter() {
            new.g[i] = self.g[i] & selected.to_u16();
        }
        new
        */
    }

    #[inline]
    fn is_supergraph(&self, other: &Self) -> bool {
        /*
        let mut res = true;
        for i in 0..16 {
            res = res && (!self.g[i] & other.g[i]) == 0;
        }
        res
        */
        unsafe {
            let a = self.to_simd();
            let b = other.to_simd();
            let r = _mm256_testc_si256(a, b);
            r != 0
        }
    }

    #[inline]
    fn union(&mut self, other: &Self) {
        // Compiler manages this one
        for i in 0..16 {
            self.g[i] = self.g[i] | other.g[i];
        }
    }

    #[inline]
    fn difference(&mut self, other: &Self) {
        // FIXME: something is wrong here
        for i in 0..16 {
            self.g[i] = self.g[i] & !other.g[i];
            self.g[i] |= (!(self.has_node(i) && other.has_node(i)) as u16) << i;
        }
    }

    #[inline]
    fn bipartite_split(&self, a: &Self::Set, b: &Self::Set) -> Self {
        /*
        let mut new = Self::empty();
        for i in a.iter() {
            new.g[i] = self.g[i] & b.to_u16();
            new.g[i] |= (self.has_node(i) as u16) << i;
        }
        for i in b.iter() {
            new.g[i] = self.g[i] & a.to_u16();
            new.g[i] |= (self.has_node(i) as u16) << i;
        }
        new
        */
        unsafe {
            let ma = _mm256_set1_epi16(a.to_u16() as i16);
            let mask_a = u16_to_mask(a.to_u16());
            let mb = _mm256_set1_epi16(b.to_u16() as i16);
            let mask_b = u16_to_mask(b.to_u16());
            let mask = _mm256_or_si256(mask_a, mask_b);
            let mask = _mm256_and_si256(mask, NODEMASK.to_simd());

            let mask_a = _mm256_and_si256(mask_a, mb);
            let mask_b = _mm256_and_si256(mask_b, ma);

            let g = self.to_simd();
            let gm = _mm256_and_si256(g, mask);
            let ga = _mm256_and_si256(g, mask_a);
            let gb = _mm256_and_si256(g, mask_b);
            let g = _mm256_or_si256(gm, ga);
            let g = _mm256_or_si256(g, gb);
            Self::from_simd(g)
        }
    }

    // Overrides

    #[inline]
    fn neighbouring(&self, nodes: &Self::Set) -> Self {
        // Using shuffle should be faster than extract
        unsafe {
            let g = self.to_simd();
            let mask = u16_to_mask(nodes.to_u16());
            let g = _mm256_and_si256(g, mask);
            let a = _mm256_extracti128_si256(g, 0);
            let b = _mm256_extracti128_si256(g, 1);
            let a = _mm_or_si128(a, b);
            let b = _mm_shuffle_epi32(a, 78);

            let a = _mm_or_si128(a, b);
            let b = _mm_shuffle_epi32(a, 229);

            let a = _mm_or_si128(a, b);
            let b = _mm_srli_epi32(a, 16);

            let a = _mm_or_si128(a, b);
            let r = _mm_extract_epi16(a, 0);

            let selection = Bitset16::from_u16(r as u16);

            self.subgraph(&selection)
        }
        /*
        unsafe {
            let g = self.to_simd();
            let mask = u16_to_mask(nodes.to_u16());
            let g = _mm256_and_si256(g, mask);
            let a = _mm256_extracti128_si256(g, 0);
            let b = _mm256_extracti128_si256(g, 1);
            let g = _mm_or_si128(a, b);
            let a = _mm_extract_epi64(g, 0);
            let b = _mm_extract_epi64(g, 1);
            let g = a | b;
            let g = (g >> 32) | g;
            let g = (g >> 16) | g;
            let selection = Bitset16::from_u16(g as u16);

            self.subgraph(&selection)
        }
        */
    }
}

impl std::fmt::Debug for Graph16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.nodes().iter().map(|u| {
            (u, self.siblings(u).iter().collect::<Vec<_>>())
        })).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn del_nodes() {
        let mut a = Graph16::complete(7);

        let mut b = a.clone();

        let mut set = Bitset16::new();
        for i in 0..2 {
            a.del_node(i);
            set.set(i);
        }
        b.del_nodes(&set);

        assert_eq!(a, b);
    }

    #[test]
    fn add_edges() {
        let mut a = Graph16::complete(7);
        a.add_node(7);

        let mut b = a.clone();
        let mut set = Bitset16::new();
        for i in 0..3 {
            a.add_edge(i, 7);
            set.set(i);
        }
        b.add_edges(7, &set);

        assert_eq!(a, b);
    }

    #[test]
    fn subgraph() {
        let g = Graph16::complete(7);

        let mut set = Bitset16::new();
        for i in 0..5 {
            set.set(i);
        }

        let subgraph = g.subgraph(&set);

        assert_eq!(Graph16::complete(5), subgraph);
    }

    #[test]
    fn is_supergraph() {
        let a = Graph16::complete(5);
        let b = Graph16::complete(7);
        assert!(b.is_supergraph(&a));
        assert!(!a.is_supergraph(&b));
        assert!(a.is_supergraph(&a));
    }

    /*
    #[test]
    fn neighbouring() {
        let mut a = Graph16::complete(1);
        let mut b = Graph16::complete(1);

        let mut set = Bitset16::new();
        for i in 1..5 {
            set.set(i);
            g.add_node(i);
            g.add_edge(0, i);
        }

        let a = Graph16::complete(5);
        let b = Graph16::complete(7);
        assert!(b.is_supergraph(&a));
        assert!(!a.is_supergraph(&b));
        assert!(a.is_supergraph(&a));
    }
    */
}
