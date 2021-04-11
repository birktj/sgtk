use crate::graph::Graph;
use crate::bitset::{Intset, Bitset};

pub fn from_graph6<G: Graph>(mut s: &str) -> G {
    let n = if s.starts_with("~~") {
        let r = s[2..8].as_bytes().iter().map(|b| b - b'?')
            .fold(0, |acc, x| {
                usize::from(x) + (acc << 6)
            });

        s = &s[8..];
        r
    } else if s.starts_with("~") {
        let r = s[2..4].as_bytes().iter().map(|b| b - b'?')
            .fold(0, |acc, x| {
                usize::from(x) + (acc << 6)
            });

        s = &s[4..];
        r
    } else {
        let r = usize::from(s.as_bytes()[0] - b'?');
        s = &s[1..];
        r
    };

    let mut graph = G::empty();
    for i in 0..n {
        graph.add_node(i);
    }

    let mut bits = s.as_bytes().iter().map(|b| b - b'?')
        .flat_map(|b| std::array::IntoIter::new([(b >> 5) & 1, (b >> 4) & 1, (b >> 3) & 1, (b >> 2) & 1, (b >> 1) & 1, b & 1]));

    for u in 1..n {
        for v in 0..u {
            if bits.next().unwrap() == 1 {
                graph.add_edge(u, v);
            }
        }
    }

    graph
}

pub fn to_graph6<G: Graph>(graph: &G) -> String {
    let mut res = String::new();
    res.push((graph.nodes().count() as u8 + b'?') as char);
    let mut bits = 0;
    let mut j = 0;
    for (i, u) in graph.nodes().iter().enumerate().skip(1) {
        for v in graph.nodes().iter().take(i) {
            bits = bits << 1;
            if graph.has_edge(u, v) {
                bits |= 1;
            }
            if j % 6 == 5 {
                res.push((bits + b'?') as char);
                bits = 0;
            }
            j += 1;
        }
    }
    if j % 6 != 0 {
        bits = bits << (6 - j % 6);
        res.push((bits + b'?') as char);
    }
    res
}

pub fn from_upper_tri<G: Graph>(mut s: &str) -> Option<G> {
    s = s.trim();
    let sn = s.split(' ').next().unwrap();
    let n = sn.parse::<usize>().unwrap();
    if n > G::MAXN {
        return None
    }
    s = s.strip_prefix(sn).unwrap();

    let mut graph = G::empty();
    for i in 0..n {
        graph.add_node(i);
    }

    let mut edges = s.chars().filter(|c| *c == '1' || *c == '0');

    for u in 0..n - 1 {
        for v in u+1..n {
            if edges.next().unwrap() == '1' {
                graph.add_edge(u, v);
            }
        }
    }
    assert!(edges.next().is_none());
    Some(graph)
}
