use crate::seq::{Seq16, SeqPermutations16};
use crate::bitset::Bitset16;
use crate::smallvec::Smallvec;
use crate::Graph16;

#[derive(Clone)]
pub struct RotationSystem16 {
    nodes: Bitset16,
    edges: [Bitset16; 16],
    order: [[u8; 16]; 16],
    order_inv: [[u8; 16]; 16],
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Face16 {
    u0: usize,
    v0: usize,
}

impl RotationSystem16 {
    pub fn simple(graph: &Graph16) -> Self {
        let nodes = graph.nodes();
        let mut edges = [Bitset16::new(); 16];
        let mut order = [[0; 16]; 16];
        let mut order_inv = [[0; 16]; 16];

        for u in graph.nodes() {
            edges[u] = graph.siblings(u);
            for (v, w) in graph.siblings(u).into_iter()
                .zip(graph.siblings(u).into_iter().cycle().skip(1))
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

    pub fn to_graph(&self) -> Graph16 {
        let mut graph = Graph16::new(0);
        for u in self.nodes {
            graph = graph.add_node(u);
            for v in self.edges[u] {
                graph = graph.add_edge(u, v);
            }
        }
        graph
    }

    pub fn genus(&self) -> usize {
        (3 + self.to_graph().edges().count() - self.nodes.count() - self.faces().count()) / 2
    }

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

    pub fn faces<'a>(&'a self) -> impl 'a + Iterator<Item = Face16> {
        let mut used = [Bitset16::new(); 16];
        let mut u = 0;
        
        std::iter::from_fn(move || {
            // TODO: use bitset?
            while u < 16 {
                if let Some(v) = self.edges[u]
                    .intersection(&used[u].invert())
                    .smallest()
                {
                    let face = Face16 { u0: u, v0: v };
                    for (ki, kj) in self.face(face) {
                        used[ki].set(kj);
                    }
                    return Some(face)
                }
                u += 1;
            }
            None
        })
    }

    pub fn face<'a>(&'a self, face: Face16) -> impl 'a + Iterator<Item = (usize, usize)> {
        let mut u = face.u0;
        let mut v = face.v0;
        let mut finished = false;
        std::iter::from_fn(move || {
            if finished {
                return None
            }

            let next_v = usize::from(self.order[v][u]);

            let old_u = u;
            u = v;
            v = next_v;

            if u == face.u0 && v == face.v0 {
                finished = true;
            }
            Some((old_u, u))
        })
    }

    pub fn face_nodes(&self, face: Face16) -> Bitset16 {
        let mut nodes = Bitset16::new();
        for (u, v) in self.face(face) {
            nodes.set(u);
            nodes.set(v);
        }
        nodes
    }

    fn insert_edge(&mut self, node: usize, after: usize, dest: usize) {
        self.edges[node].set(dest);
        let k = self.order[node][after];
        self.order[node][dest] = k;
        self.order_inv[node][dest] = after as u8;
        self.order_inv[node][usize::from(k)] = dest as u8;
        self.order[node][after] = dest as u8;
    }

    fn insert_edge_any(&mut self, node: usize, dest: usize) {
        self.nodes.set(node);
        self.nodes.set(dest);
        if let Some(i) = self.edges[node].smallest() {
            self.insert_edge(node, i, dest);
        } else {
            self.insert_edge(node, dest, dest);
        }
    }

    pub fn embed_free_edge(&mut self, u: usize, v: usize) {
        self.insert_edge_any(u, v);
        self.insert_edge_any(v, u);
    }

    pub fn embed_bisecting_path(&mut self, face: Face16, path: &Seq16) -> [Face16; 2] {
        let start     = path.first().unwrap();
        let start_snd = usize::from(path[1]);
        let end       = path.last().unwrap();
        let end_snd   = usize::from(path[path.len()-2]);

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

        self.insert_edge(start, usize::from(self.order_inv[start][start_u.unwrap()]), start_snd);
        self.insert_edge(end, end_u.unwrap(), end_snd);
        if start_snd != end {
            self.insert_edge_any(start_snd, start);
            self.insert_edge_any(end_snd, end);
        }

        if path.len() > 2 {
            for (u, v) in (&path.slice()[1..path.len()-2]).iter().zip(path.iter().skip(2)) {
                self.embed_free_edge(usize::from(*u), usize::from(*v));
            }
        }

        // This should in theory give the two faces on each side of the bisecting path
        [Face16 { u0: start, v0: start_snd }, Face16 { u0: start_snd, v0: start }]
    }
}

impl std::fmt::Debug for RotationSystem16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.nodes.into_iter().map(|u| {
            (u, self.siblings(u).collect::<Vec<_>>())
        })).finish()
    }
}
