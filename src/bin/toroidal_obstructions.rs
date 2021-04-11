use sgtk::graph::{minors, Graph64};
use sgtk::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "toroidal_obstructions", about = "Tool to search for toroidal obstructions.")]
struct Opt {
    /// Number of random graphs to test
    #[structopt(short, long, default_value="100")]
    count: u64,
    /// Size of random graphs
    #[structopt(short, default_value="63")]
    n: usize,
    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

fn find_toroidal_obstruction(graph: Graph64) -> Graph64 {
    for minor in minors(&graph).filter(|minor| minor.is_connected()) {
        if sgtk::toroidal::find_embedding(&minor).is_none() {
            return find_toroidal_obstruction(minor)
        }
    }

    graph.to_canonical()
}

struct Stats {
    num_toroidal: u64,
    num_disconnected: u64,
    num_obstructions: u64,
    count_sizes: BTreeMap<usize, u64>,
    obstructions: HashMap<Graph64, u64>,
}

fn main() {
    let opt = Opt::from_args();

    let mut output = opt.output.map(|path| std::fs::File::create(path).unwrap())
        .unwrap_or_else(|| std::fs::File::create("/dev/stdout").unwrap());


    let mut stats = Stats {
        num_toroidal: 0,
        num_disconnected: 0,
        num_obstructions: 0,
        count_sizes: BTreeMap::new(),
        obstructions: HashMap::new(),
    };

    for _ in 0..opt.count {
        let graph: Graph64 = sgtk::random::graph(opt.n);
        if !graph.is_connected() {
            stats.num_disconnected += 1;
            continue
        }

        if sgtk::toroidal::find_embedding(&graph).is_none() {
            stats.num_obstructions += 1;
            let obstruction = find_toroidal_obstruction(graph);
            *stats.count_sizes.entry(obstruction.nodes().count()).or_insert(0) += 1;
            *stats.obstructions.entry(obstruction).or_insert(0) += 1;
            write!(output, "{}\n", sgtk::parse::to_graph6(&obstruction)).unwrap();
        } else {
            stats.num_toroidal += 1;
        }
    }
    output.flush().unwrap();

    eprintln!("{} random graphs tested", opt.count);
    eprintln!("{} graphs where discarded, {} disconnected and {} toroidal",
        stats.num_disconnected + stats.num_toroidal,
        stats.num_disconnected,
        stats.num_toroidal);
    eprintln!("{} obstructions where found", stats.num_obstructions);
    for (i, n) in &stats.count_sizes {
        eprintln!("{} obstructions with {} nodes", n, i);
    }
}
