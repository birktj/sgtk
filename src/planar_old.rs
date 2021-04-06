use crate::seq::{Seq16, SeqPermutations};
use crate::bitset::{Bitset, Bitset16};
use crate::Graph16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RotationSystem16 {
    n: usize,
    edges: [Seq16; 16],
}

pub struct Face16<'a> {
    u0: usize,
    v0: usize,
    rotation_system: &'a RotationSystem16,
}

impl<'a> Face16<'a> {
    pub fn iter<'b>(&'b self) -> impl 'b + Iterator<Item = (usize, usize)> {
        let mut u = self.u0;
        let mut v = self.v0;
        let mut finished = false;
        std::iter::from_fn(move || {
            if finished {
                return None
            }

            let next_v = self.rotation_system.edges[v].iter()
                .cycle() // Chain may be better?
                .skip_while(|k| usize::from(**k) != u)
                .skip(1).next()
                .expect("Graph is not consistent");

            let old_u = u;
            u = v;
            v = usize::from(*next_v);

            if u == self.u0 && v == self.v0 {
                finished = true;
            }
            Some((old_u, u))
        })
    }

    pub fn nodes(&self) -> Bitset16 {
        let mut nodes = Bitset16::new();
        for (u, v) in self.iter() {
            nodes.set(u);
            nodes.set(v);
        }
        nodes
    }
}

impl RotationSystem16 {
    pub fn simple(graph: &Graph16) -> Self {
        let mut edges = [Seq16::new(); 16];
        let n = graph.nodes().count();
        
        for u in graph.nodes() {
            for v in graph.siblings(u).into_iter().filter(|v| *v != u) {
                edges[u].push(v);
            }
        }

        Self {
            n,
            edges
        }
    }

    pub fn enumerate(graph: &Graph16) -> RotationSystemEnumerate16 {
        let curr = RotationSystem16::simple(graph);
        let mut permutations = [SeqPermutations::empty(); 16];
        if curr.n > 0 {
            permutations[0] = curr.edges[0].permutations();
        }

        RotationSystemEnumerate16 {
            curr,
            permutations
        }
    }

    fn to_graph(&self) -> Graph16 {
        let mut graph = Graph16::new(self.n);
        for u in 0..self.n {
            for v in self.edges[u].iter() {
                graph.add_edge(u, usize::from(*v));
            }
        }

        graph
    }

    pub fn genus(&self) -> usize {
        (3 + self.to_graph().edges().count() - self.n - self.count_faces()) / 2
    }

    fn find_face<F: FnMut(usize, usize)>(&self, u0: usize, v0: usize, mut f: F) {
        //let mut seq = Seq16::new();
        let mut u = u0;
        let mut v = v0;
        //seq.push(u);
        //seq.push(v);
            
        loop {
            f(u, v);
            let next_v = self.edges[v].iter().cycle() // Chain may be better?
                .skip_while(|k| usize::from(**k) != u)
                .skip(1).next()
                .expect("Graph is not consistent");

            u = v;
            v = usize::from(*next_v);
            //seq.push(v);
            if u == u0 && v == v0 {
                break
            }
        }

        //seq
    }

    // FIXME: there must be a better way to do this?
    pub fn faces<'a>(&'a self) -> impl 'a + Iterator<Item = Face16<'a>> {
        let mut used = [Bitset16::new(); 16];
        let mut u = 0;
        let mut v_iter = self.edges[u].iter();
        
        std::iter::from_fn(move || {
            while u < self.n {
                while let Some(v) = v_iter.next() {
                    if used[u].get(usize::from(*v)) {
                        continue
                    }
                    self.find_face(u, usize::from(*v), |ki, kj| {
                        used[ki].set(kj);
                    });
                    return Some(Face16 {
                        u0: u,
                        v0: usize::from(*v),
                        rotation_system: &self,
                    })
                }
                u += 1;
                if u < self.n {
                    v_iter = self.edges[u].iter();
                }
            }
            None
        })
    }

    pub fn faces_nodes<'a>(&'a self) -> impl 'a + Iterator<Item = Bitset16> {
        let mut used = [Bitset16::new(); 16];
        let mut u = 0;
        let mut v_iter = self.edges[u].iter();
        
        std::iter::from_fn(move || {
            while u < self.n {
                while let Some(v) = v_iter.next() {
                    if used[u].get(usize::from(*v)) {
                        continue
                    }
                    let mut res = Bitset16::new();
                    self.find_face(u, usize::from(*v), |ki, kj| {
                        used[ki].set(kj);
                        res.set(ki);
                        res.set(kj);
                    });
                    return Some(res)
                }
                u += 1;
                if u < self.n {
                    v_iter = self.edges[u].iter();
                }
            }
            None
        })
    }

    pub fn count_faces(&self) -> usize {
        let mut used = [Bitset16::new(); 16];
        let mut count = 0;

        for u in 0..self.n {
            for v in self.edges[u].iter() {
                if !used[u].get(usize::from(*v)) {
                    self.find_face(u, usize::from(*v), |ki, kj| {
                        used[ki].set(kj);
                    });
                    /*
                    let face = self.find_face(u, usize::from(*v));
                    for (ki, kj) in face.iter().zip(face.iter().skip(1)) {
                        used[usize::from(*ki)].set(usize::from(*kj));
                    }*/
                    count += 1;
                }
            }
        }

        std::cmp::max(1, count)
    }
}

pub struct RotationSystemEnumerate16 {
    curr: RotationSystem16,
    permutations: [SeqPermutations<16>; 16],
}

impl Iterator for RotationSystemEnumerate16 {
    type Item = RotationSystem16;

    fn next(&mut self) -> Option<RotationSystem16> {
        for i in (0..self.curr.n).rev() {
            if let Some(new) = self.permutations[i].next() {
                self.curr.edges[i] = new;
                for j in i+1..self.curr.n {
                    self.permutations[j] = self.curr.edges[j].permutations();
                    self.curr.edges[j] = self.permutations[j].next().unwrap();
                }
                return Some(self.curr)
            }
        }

        None
    }
}

pub fn check_genus_connected(genus: usize, graph: Graph16) -> Option<RotationSystem16> {
    struct Searcher {
        max_genus: usize,
        graph: Graph16,
        curr: Graph16,
        rotation: RotationSystem16,
    }

    impl Searcher {
        fn search(&mut self, u: usize, v: usize) -> bool {
            //dbg!(u, v);
            if u == self.rotation.n {
                return true
            }
            if v == self.rotation.n {
                return self.search(u+1, u+2)
            }
            if !self.graph.has_edge(u, v) {
                return self.search(u, v+1)
            }
            if self.curr.has_edge(u, v) {
                return self.search(u, v+1)
            }
            self.curr.add_edge(u, v);
            let u_edges = self.rotation.edges[u];
            let v_edges = self.rotation.edges[v];
            //dbg!(self.rotation.edges[u]);
            //dbg!(self.rotation.edges[v]);

            self.rotation.edges[u].push(v);
            for i in (0..u_edges.len()+1).rev() {
                if i < u_edges.len() {
                    self.rotation.edges[u].swap(i, i+1);
                }
                self.rotation.edges[v].push(u);
                for j in (0..v_edges.len()+1).rev() {
                    //dbg!(i, j);
                    if j < v_edges.len() {
                        self.rotation.edges[v].swap(j, j+1);
                    }
                    //dbg!(self.rotation.edges[u]);
                    //dbg!(self.rotation.edges[v]);
                    if self.rotation.genus() <= self.max_genus && self.search(u, v+1) {
                        return true
                    }
                }

                self.rotation.edges[v] = v_edges;
            }
            self.rotation.edges[u] = u_edges;
            self.curr.del_edge(u, v);
            false
        }
    }

    let spanning_tree = graph.spanning_tree();

    for rotation in RotationSystem16::enumerate(&spanning_tree)  {
        let mut searcher = Searcher {
            max_genus: genus,
            graph,
            curr: spanning_tree,
            rotation,
        };

        if searcher.search(0, 1) {
            return Some(searcher.rotation)
        }
    }
    None
}

pub fn dmp(graph: &Graph16) -> Option<RotationSystem16> {
    // Find cycle
    let mut h = if let Some(c) = graph.cycle() {
        c
    } else {
        // No cycle, graph must be a tree. Any embedding should be valid
        // FIXME: is this true?
        return Some(RotationSystem16::simple(&graph))
    };
    

    let mut embedding = RotationSystem16::simple(&h);
    embedding.n = graph.nodes().count();

    loop {
        //dbg!(h);
        //dbg!(embedding);
        let bridges = graph.edges().filter(|(u, v)| {
            h.has_node(*u) && h.has_node(*v) && !h.has_edge(*u, *v)
        }).map(|(u, v)| {
            let mut g = Graph16::new(0);
            g.add_node(u);
            g.add_node(v);
            g.add_edge(u, v);
            g
        }).chain(graph.subgraph(h.nodes().invert()).components()
            .map(|c| {
                let mut newc = c;
                for (u, v) in graph.edges() {
                    if (h.has_node(u) && c.has_node(v)) || (h.has_node(v) && c.has_node(u)) {
                        newc.add_node(u);
                        newc.add_node(v);
                        newc.add_edge(u, v);
                    }
                }

                newc
            }));

        let mut bridge = None;

        for b in bridges {
            let attachments = h.nodes().intersection(&b.nodes());
            //dbg!(b);
            //dbg!(attachments);

            let num = embedding.faces_nodes()
                .filter(|face| face.is_superset(&attachments))
                .count();

            //dbg!(num);

            // Bridge with no admissible faces
            if num == 0 {
                return None
            }

            if let Some((ref mut bridge, ref mut bridge_num)) = bridge.as_mut() {
                if *bridge_num > num {
                    *bridge_num = num;
                    *bridge = b;
                }
            } else {
                bridge = Some((b, num));
            }
        }

        if let Some((bridge, _)) = bridge {
            //dbg!(bridge);
            let mut attachments = bridge.nodes().intersection(&h.nodes());
            //dbg!(attachments);

            if attachments.count() == 1 {
                let u = attachments.smallest().unwrap();
                let mut siblings = bridge.siblings(u);
                siblings.clear(u);
                let v = siblings.smallest().unwrap();
                h.add_node(u);
                h.add_node(v);
                h.add_edge(u, v);
                embedding.edges[u].push(v);
                embedding.edges[v].push(u);
                continue
            }

            let face = embedding.faces()
                .filter(|face| face.nodes().is_superset(&attachments))
                .next().unwrap();

            //dbg!(face.iter().collect::<Vec<_>>());

            let start = attachments.smallest().unwrap();
            attachments.clear(start);

            let mut path = bridge.path(start, attachments).unwrap();
            //dbg!(path);
            let start_snd = usize::from(path[1]);
            let end = path.pop().unwrap();
            let end_snd = path.last().unwrap();

            let mut start_i = None;
            let mut end_i = None;

            for (u, v) in face.iter() {
                if start_i.is_none() && u == start {
                    start_i = embedding.edges[u].iter()
                        .position(|k| usize::from(*k) == v);
                        //.map(|i| (i + 1) % embedding.edges[u].len());
                }
                if end_i.is_none() && v == end {
                    end_i = embedding.edges[v].iter()
                        .position(|k| usize::from(*k) == u)
                        .map(|i| (i + 1) % embedding.edges[u].len());
                }
            }

            embedding.edges[start].insert(start_i.unwrap(), start_snd);
            if end != start_snd {
                embedding.edges[start_snd].push(start);
            }

            for (u, v) in path.iter().skip(1).zip(path.iter().skip(2)) {
                let u = usize::from(*u);
                let v = usize::from(*v);
                embedding.edges[u].push(v);
                embedding.edges[v].push(u);
                h.add_node(u);
                h.add_node(v);
                h.add_edge(u, v);
            }

            embedding.edges[end].insert(end_i.unwrap(), end_snd);
            if end != start_snd {
                embedding.edges[end_snd].push(end);
            }

            h.add_node(start);
            h.add_node(start_snd);
            h.add_edge(start, start_snd);
            h.add_node(end);
            h.add_node(end_snd);
            h.add_edge(end_snd, end);

            // TODO
            //embedding.embed(path);

            //h = h.union(&dfs.path);
        } else {
            // No bridges
            return Some(embedding)
        }
    }
}
