use sgtk::graph::{minors, subgraphs, Graph32};
use sgtk::prelude::*;
use console::style;
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub mod splitdel;
pub mod editsearch;

pub fn is_obstruction<G: Graph>(graph: &G) -> bool {
    for u in graph.nodes().iter() {
        if graph.siblings(u).count() < 3 {
            return false
        }
    }
    if sgtk::toroidal::find_embedding(graph).is_some() {
        return false
    }
    for subgraph in subgraphs(graph) {
        if sgtk::toroidal::find_embedding(&subgraph).is_none() {
            return false
        }
    }

    true
}

pub fn is_minor_obstruction<G: Graph>(graph: &G) -> bool {
    for (u, v) in graph.edges() {
        let mut graph = graph.to_owned();
        graph.contract_edge(u, v);
        if sgtk::toroidal::find_embedding(&graph).is_none() {
            return false
        }
    }

    true
}


pub struct Stats {
    pub graphs: HashSet<Graph32>,
    unique_count: Vec<usize>,
    found_count: Vec<usize>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            graphs: HashSet::new(),
            found_count: vec![0; 32],
            unique_count: vec![0; 32],
        }
    }

    pub fn add_graph(&mut self, graph: Graph32) {
        self.found_count[graph.nodes().count()] += 1;
        if !self.graphs.contains(&graph) {
            self.unique_count[graph.nodes().count()] += 1;
        }
        self.graphs.insert(graph);
    }

    pub fn print(&self, title: &str) {
        eprintln!("{}\n", style(title).bold());

        eprintln!("{:>10} {:>10} {:>10}", style("Vertices").bold(), style("Found").bold(), style("Unique").bold());

        for (v, (fc, uc)) in self.found_count.iter().zip(self.unique_count.iter()).enumerate() {
            if *fc > 0 || *uc > 0 {
                eprintln!("{:>10} {:>10} {:>10}", v, fc, uc);
            }
        }
        
        let fc_tot: usize = self.found_count.iter().sum();
        let uc_tot: usize = self.unique_count.iter().sum();
        eprintln!("{:>10} {:>10} {:>10}", "total", fc_tot, uc_tot);

        eprintln!("");
    }

    pub fn print_edges(&self, title: &str) {
        let min_edges = self.graphs.iter().map(|g| g.edges_count()).min().unwrap();
        let max_edges = self.graphs.iter().map(|g| g.edges_count()).max().unwrap();

        let mut edge_count = vec![0; max_edges+1];

        eprintln!("{}\n", style(title).bold());

        eprint!("{:>8} ", style("Vertices").bold());
        for e in min_edges ..= max_edges {
            eprint!("{:>5} ", style(&e.to_string()).bold());
        }
        eprintln!("{:>6}", "total");

        for (v, fc) in self.unique_count.iter().enumerate() {
            if *fc == 0 {
                continue
            }
            eprint!("{:>8} ", v);
            for e in min_edges ..= max_edges {
                let count = self.graphs.iter().filter(|g| g.nodes().count() == v && g.edges_count() == e).count();
                edge_count[e] += count;
                if count > 0 {
                    eprint!("{:>5} ", count);
                } else {
                    eprint!("{:>5} ", "-");
                }
            }
            eprintln!("{:>6}", fc);
        }
        
        let fc_tot: usize = self.unique_count.iter().sum();
        eprint!("{:>8} ", "total");
        for e in min_edges ..= max_edges {
            eprint!("{:>5} ", edge_count[e]);
        }
        eprintln!("{:>6}", fc_tot);

        eprintln!("");
    }
}

pub struct TimingStats {
    times: Vec<Vec<Duration>>,
}

impl TimingStats {
    pub fn new() -> Self {
        Self {
            times: vec![Vec::new(); 32],
        }
    }

    pub fn time_graph<G: Graph, R, F: FnOnce(G) -> R>(&mut self, graph: G, f: F) -> R {
        let nodes = graph.nodes().count();
        let now = Instant::now();
        let res = f(graph);
        let duration = now.elapsed();
        self.times[nodes].push(duration);
        res
    }

    pub fn print(&self, title: &str) {
        eprintln!("{}\n", style(title).bold());
        eprintln!("{:>10} {:>10} {:>10} {:>10} {:>10}", style("Vertices").bold(), style("Avg").bold(), style("Max").bold(), style("Min"), style("Total"));
        for (n, times) in self.times.iter().enumerate() {
            if !times.is_empty() {
                let mut avg = Duration::new(0,0);
                let mut max = Duration::new(0,0);
                let mut min = Duration::new(1000,0);
                let mut total = Duration::new(0,0);
                for time in times {
                    avg += *time;
                    max = std::cmp::max(max, *time);
                    min = std::cmp::min(min, *time);
                    total += *time;
                }
                avg /= times.len() as u32;
                eprintln!("{:>10} {:>10} {:>10} {:>10} {:>10}", n, avg.as_micros(), max.as_micros(), min.as_micros(), total.as_micros());
            }
        }
    }
}

