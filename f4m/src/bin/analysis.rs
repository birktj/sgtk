use sgtk::graph::{minors, subgraphs, Graph32};
use sgtk::prelude::*;
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use std::io::Write;
use std::time::{Instant, Duration};
use structopt::StructOpt;
use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use console::style;
use f4m::{Stats, TimingStats};

#[derive(StructOpt, Debug)]
#[structopt(name = "f4m-analysis", about = "Tool to analyse toroidal obstructions.")]
struct Opt {
    #[structopt(long)]
    print_edge_count: bool,
    /// Known torus obstrutions in upper triangle format
    #[structopt(short, long, parse(from_os_str))]
    known_obstructions: Option<PathBuf>,
    /// List of new torus obstructions
    #[structopt(parse(from_os_str))]
    new_obstructions: Vec<PathBuf>,
    /// File to write unkown obstructions to
    #[structopt(long, parse(from_os_str))]
    unknown_file: Option<PathBuf>,
    /// Check if the new obstructions really are obstructions
    #[structopt(short, long)]
    check: bool,
    /// Check if the new obstructions are minor order
    #[structopt(long)]
    check_minor: bool,
    /// Convert new obstructions to canonical form
    #[structopt(long)]
    to_canonical: bool,
    /// Do timing tests
    #[structopt(long)]
    time: bool,
    /// Input graph is in upper triangle format
    #[structopt(long)]
    triangle: bool,
}

fn is_obstruction(graph: &Graph32) -> bool {
    for u in graph.nodes() {
        if graph.siblings(u).count() < 3 {
            return false
        }
    }
    if sgtk::toroidal::find_embedding(graph).is_some() {
        return false
    }
    for (u, v) in graph.edges() {
        let mut subgraph = graph.clone();
        subgraph.del_edge(u, v);
        if sgtk::toroidal::find_embedding(&subgraph).is_none() {
            return false
        }
    }

    true
}

fn is_minor_obstruction(graph: &Graph32) -> bool {
    for (u, v) in graph.edges() {
        let mut graph = graph.to_owned();
        graph.contract_edge(u, v);
        if sgtk::toroidal::find_embedding(&graph).is_none() {
            return false
        }
    }

    true
}


fn main() -> Result<()> {
    let opt = Opt::from_args();
    let progress_style = ProgressStyle::default_bar()
        .template("{prefix}{bar:40.cyan/blue} {pos:>7}/{len:7} {elapsed} elapsed, est {eta} left {msg}")
        .progress_chars("#> ");

    let mut known_obstructions = HashSet::new();
    if let Some(known) = opt.known_obstructions {
        eprintln!("Loading known obstructions");
        let file = std::fs::read_to_string(&known)
            .with_context(|| format!("Failed to read known obstrucions from {:?}", &known))?;
        let num = file.lines().count();

        let bar = ProgressBar::new(num as u64);
        bar.set_style(progress_style.clone());
        for line in file.lines() {
            let graph = sgtk::parse::from_upper_tri::<Graph32>(line)
                .ok_or(anyhow!("Known obstruction contains more than 32 vertices"))?;
            known_obstructions.insert(graph.to_canonical());
            bar.inc(1);
        }
        bar.finish();
    }

    eprintln!("Loading found obstructions:");
    let mut new_minors = Stats::new();
    let mut new_obstructions = Stats::new();
    let mut unknown_obstructions = Stats::new();
    let mut unknown_minors = Stats::new();

    let mut check_obstructions_timer = TimingStats::new();

    let mut graph_is_obstruction = HashMap::new();
    let mut graph_is_minor = HashMap::new();

    let num_len = opt.new_obstructions.len().to_string().len();
    for (i, obstr_file) in opt.new_obstructions.iter().enumerate() {
        let file = std::fs::read_to_string(obstr_file)
            .with_context(|| format!("Failed to read found obstrucions from {:?}", &obstr_file))?;
        let num = file.lines().count();
        let bar = ProgressBar::new(num as u64);
        bar.set_style(progress_style.clone());
        bar.set_prefix(&format!("[{:width$}/{:width$}] ", i+1, opt.new_obstructions.len(), width = num_len));
        for line in file.lines() {
            let mut graph = if opt.triangle {
                sgtk::parse::from_upper_tri::<Graph32>(line)
                    .ok_or(anyhow!("Known obstruction contains more than 32 vertices"))?
            } else {
                sgtk::parse::from_graph6::<Graph32>(line)
            };
            if opt.to_canonical {
                graph = graph.to_canonical();
            }
            
            if !graph_is_obstruction.contains_key(&graph) {
                if opt.check {
                    let is_obs = check_obstructions_timer.time_graph(graph.clone(), |g| is_obstruction(&g));
                    graph_is_obstruction.insert(graph, is_obs);
                } else {
                    graph_is_obstruction.insert(graph, true);
                }
                if opt.check_minor && !graph_is_minor.contains_key(&graph) {
                    graph_is_minor.insert(graph, is_minor_obstruction(&graph));
                }
            }

            if graph_is_obstruction[&graph] {
                new_obstructions.add_graph(graph);
                if !known_obstructions.contains(&graph) {
                    unknown_obstructions.add_graph(graph);
                }
                if opt.check_minor && graph_is_minor[&graph] {
                    new_minors.add_graph(graph);
                    if !known_obstructions.contains(&graph) {
                        unknown_minors.add_graph(graph);
                    }
                }
            }
            bar.inc(1);
        }
        bar.finish();
    }

    eprintln!("\n");

    new_obstructions.print("Found obstructions");
    if opt.check_minor {
        new_minors.print("Found minors");
    }

    if !unknown_obstructions.graphs.is_empty() {
        if opt.print_edge_count {
            unknown_obstructions.print_edges("Unknown obstructions");
        } else {
            unknown_obstructions.print("Unknown obstructions");
        }
    }
    if !unknown_minors.graphs.is_empty() {
        if opt.print_edge_count {
            unknown_minors.print_edges("Unknown minors");
        } else {
            unknown_minors.print("Unknown minors");
        }
    }

    if opt.time {
        check_obstructions_timer.print("Time to check obstructions");
    }

    if !unknown_obstructions.graphs.is_empty() {
        if let Some(path) = opt.unknown_file {
            let mut file = std::fs::File::create(&path)
                .with_context(|| format!("Cannot create file to write unknowns to at {:?}", path))?;
            for obstruction in &unknown_obstructions.graphs {
                writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                    .context("Cannot write new unknown obstruction")?;
            }
            file.flush()?;
        }
    }

    Ok(())
}
