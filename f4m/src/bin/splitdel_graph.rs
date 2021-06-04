use sgtk::graph::{minors, subgraphs, Graph32};
use sgtk::prelude::*;
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use std::io::Write;
use structopt::StructOpt;
use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use console::style;
use f4m::Stats;

struct SCC {
    graph: HashMap<Graph32, Vec<Graph32>>,
    component_map: HashMap<Graph32, usize>,
    components: HashMap<usize, Vec<Graph32>>,
    component_graph: HashMap<usize, HashSet<usize>>,
    component_graph_rev: HashMap<usize, HashSet<usize>>,
    in_stack: HashSet<Graph32>,
    stack: Vec<Graph32>,
    val: usize,
    start_vals: HashMap<Graph32, usize>,
    low_vals: HashMap<Graph32, usize>,
}

impl SCC {
    fn new(graph: HashMap<Graph32, Vec<Graph32>>) -> SCC {
        let elements = graph.keys().cloned().collect::<Vec<_>>();
        let mut scc = SCC {
            graph,
            component_map: HashMap::new(),
            components: HashMap::new(),
            component_graph: HashMap::new(),
            component_graph_rev: HashMap::new(),
            in_stack: HashSet::new(),
            stack: Vec::new(),
            val: 0,
            start_vals: HashMap::new(),
            low_vals: HashMap::new(),
        };

        for e in elements {
            if !scc.start_vals.contains_key(&e) {
                scc.search(e);
            }
        }
        
        scc
    }

    fn search(&mut self, u: Graph32) {
        if self.start_vals.contains_key(&u) {
            return
        }
        let start_val = self.val;
        let mut curr_val = start_val;
        self.start_vals.insert(u.clone(), curr_val);
        self.low_vals.insert(u.clone(), curr_val);
        self.val += 1;
        self.stack.push(u.clone());
        self.in_stack.insert(u.clone());

        if let Some(vs) = self.graph.get(&u).cloned() {
            for v in &vs {
                if !self.start_vals.contains_key(&v) {
                    self.search(v.clone());
                    curr_val = std::cmp::min(curr_val, self.low_vals[&v]);
                } else if self.in_stack.contains(&v) {
                    curr_val = std::cmp::min(curr_val, self.start_vals[&v]);
                }
                self.low_vals.insert(u.clone(), curr_val);
            }
        }

        if start_val == curr_val {
            //let r = dfs(&self.graph, u.clone());
            let mut comp = Vec::new();
            let mut edges = HashSet::new();
            while let Some(v) = self.stack.pop() {
                //assert!(r.contains(&v));
                self.in_stack.remove(&v);
                self.component_map.insert(v.clone(), start_val);
                comp.push(v);

                if v == u {
                    break
                }
            }
            if comp.len() > 1 {
                //assert_eq!(r, dfs(&self.graph, comp[1].clone()));
            }
            for v in &comp {
                if let Some(ws) = self.graph.get(v) {
                    for w in ws {
                        if self.component_map[w] == start_val {
                            continue
                        }
                        edges.insert(self.component_map[w]);
                        self.component_graph_rev.entry(self.component_map[w])
                            .or_insert(HashSet::new())
                            .insert(start_val);
                    }
                }
            }
            //assert!(dfs(&self.graph, u.clone()).len() >= comp.len());
            comp.sort_by_key(|a| a.nodes().count());
            self.components.insert(start_val, comp);
            self.component_graph.insert(start_val, edges);
            self.component_graph_rev.entry(start_val)
                .or_insert(HashSet::new());
        }
    }
}

fn dfs<V: std::hash::Hash + Eq + Clone>(graph: &HashMap<V, Vec<V>>, start: V) -> HashSet<V> {
    let mut visited = HashSet::new();
    let mut queue = Vec::new();
    queue.push(start);

    while let Some(u) = queue.pop() {
        if visited.contains(&u) {
            continue
        }
        //unvisited.remove(&u);
        visited.insert(u.clone());

        if let Some(vs) = graph.get(&u) {
            for v in vs {
                if !visited.contains(v) {
                    queue.push(v.clone());
                }
            }
        }
    }

    visited
}

#[derive(StructOpt, Debug)]
#[structopt(name = "f4m-splitdel-graph", about = "Tool to analyse a split-delete graph.")]
struct Opt {
    /// File to store leaf nodes in
    #[structopt(long,parse(from_os_str))]
    leaf_file: Option<PathBuf>,
    /// Start graph to compute set from
    #[structopt(long)]
    start_graph: Option<String>,
    /// The files with the splitdelete graph
    #[structopt(parse(from_os_str))]
    splitdel_graphs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let progress_style = ProgressStyle::default_bar()
        .template("{prefix}{bar:40.cyan/blue} {pos:>7}/{len:7} {elapsed} elapsed, est {eta} left {msg}")
        .progress_chars("#> ");
    
    let mut splitdel_graph = HashMap::new();
    let mut splitdel_graph_rev = HashMap::new();
    let mut unvisited = HashSet::new();
    let mut obstructions = Stats::new();

    eprintln!("Loading split-delete graph");
    for path in &opt.splitdel_graphs {
        let file = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read split-delete graph from {:?}", path))?;
        for line in file.lines() {
            let mut words = line.split(' ');
            let u = words.next()
                .ok_or(anyhow!("Split-delete graph file contains empty line"))?;
            let u: Graph32 = sgtk::parse::from_graph6(&u);
            let vs = words.map(|w| sgtk::parse::from_graph6(&w))
                .filter(|v| v != &u)
                .collect::<Vec<Graph32>>();
            
            for v in &vs {
                splitdel_graph_rev.entry(v.clone()).or_insert(Vec::new())
                    .push(u.clone());
            }

            obstructions.add_graph(u.clone());
            unvisited.insert(u.clone());
            splitdel_graph.insert(u, vs);
        }
    }

    eprintln!("Computing SCC's");

    let scc = SCC::new(splitdel_graph.clone());

    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    let mut leafs = Stats::new();
    let mut leaf_reachable = Stats::new();
    let mut cycle_reachable = Stats::new();
    let mut cycle_nodes = Stats::new();
    let mut components = Stats::new();

    for (u, _) in &scc.component_graph {
        components.add_graph(scc.components[u].first().unwrap().clone());
    }

    for (u, vs) in &scc.component_graph_rev {
        if vs.is_empty() {
            queue.push(*u);
            leafs.add_graph(scc.components[u].first().unwrap().clone());
        }
    }

    while let Some(u) = queue.pop() {
        if visited.contains(&u) {
            continue
        }
        //unvisited.remove(&u);
        visited.insert(u.clone());

        for v in &scc.component_graph[&u] {
            if !visited.contains(v) {
                queue.push(*v);
                leaf_reachable.add_graph(scc.components[v].first().unwrap().clone());
            }
        }
    }

    /*
    while let Some(mut u) = unvisited.iter().next().cloned() {
        // Find a cycle
        let mut prev = HashSet::new();
        prev.insert(u.clone());
        queue.push(u.clone());

        while let Some(v) = queue.pop() {
            if prev.contains(&v) {
                u = v;
                break;
            }

            if !splitdel_graph_rev.contains_key(&u) {
                continue
            }

            for w in &splitdel_graph_rev[&v] {
                queue.push(w.clone());
            }
        }

        cycle_nodes.add_graph(u.clone());
        queue.push(u);

        while let Some(u) = queue.pop() {
            if visited.contains(&u) {
                continue
            }
            unvisited.remove(&u);
            visited.insert(u.clone());
            cycle_reachable.add_graph(u.clone());

            if !splitdel_graph.contains_key(&u) {
                continue
            }

            for v in &splitdel_graph[&u] {
                if !visited.contains(v) {
                    queue.push(v.clone());
                }
            }
        }
    }
    */

    components.print("Components");
    leafs.print("Leafs");
    leaf_reachable.print("Leaf reachable");
    cycle_nodes.print("Cycle nodes");
    cycle_reachable.print("Cycle reachable");
    obstructions.print("Obstructions");

    if let Some(start) = opt.start_graph {
        let graph = sgtk::parse::from_graph6(&start);
        let mut reachable = Stats::new();
        for g in dfs(&splitdel_graph, graph) {
            reachable.add_graph(g);
        }
        reachable.print("Reachable from start graph");
    }

    /*
    for obstruction in splitdel_graph.keys() {
        if obstruction.nodes().count() != 8 {
            continue
        }
        let comp = scc.component_map[obstruction];
        let mut stat = Stats::new();
        for g in &scc.components[&comp] {
            stat.add_graph(g.clone());
        }
        stat.print("Leaf component obstructions");
    }
    */

    if let Some(path) = opt.leaf_file {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write leafs to at {:?}", path))?;
        for obstruction in &leafs.graphs {
            writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                .context("Cannot write new unknown obstruction")?;
        }
        file.flush()?;
    }

    Ok(())
}
