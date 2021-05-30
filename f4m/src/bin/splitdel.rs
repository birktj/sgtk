use sgtk::graph::{minors, subgraphs, Graph32};
use sgtk::prelude::*;
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use std::io::Write;
use structopt::StructOpt;
use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use console::style;

#[derive(StructOpt, Debug)]
#[structopt(name = "f4m-splitdel", about = "Tool to search for toroidal obstructions with split-delete.")]
struct Opt {
    /// File to write unkown obstructions to
    #[structopt(long, parse(from_os_str))]
    unknown_file: Option<PathBuf>,
    #[structopt(short,long, parse(from_os_str))]
    start_file: Option<PathBuf>,
    /// Known torus obstrutions
    #[structopt(parse(from_os_str))]
    known: PathBuf,
}

fn is_obstruction(graph: &Graph32) -> bool {
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

struct Stats {
    graphs: HashSet<Graph32>,
    unique_count: Vec<usize>,
    found_count: Vec<usize>,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            graphs: HashSet::new(),
            found_count: vec![0; 32],
            unique_count: vec![0; 32],
        }
    }

    fn add_graph(&mut self, graph: Graph32) {
        self.found_count[graph.nodes().count()] += 1;
        if !self.graphs.contains(&graph) {
            self.unique_count[graph.nodes().count()] += 1;
        }
        self.graphs.insert(graph);
    }

    fn print(&self, title: &str) {
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
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let progress_style = ProgressStyle::default_bar()
        .template("{prefix}{bar:40.cyan/blue} {pos:>7}/{len:7} {elapsed} elapsed, est {eta} left {msg}")
        .progress_chars("#> ");

    let mut known_obstructions = HashSet::new();
    let mut obstructions = HashSet::new();
    eprintln!("Loading known obstructions");
    let file = std::fs::read_to_string(&opt.known)
        .with_context(|| format!("Failed to read known obstrucions from {:?}", &opt.known))?;
    let num = file.lines().count();

    let bar = ProgressBar::new(num as u64);
    bar.set_style(progress_style.clone());
    for line in file.lines() {
        let graph = sgtk::parse::from_upper_tri::<Graph32>(line)
            .ok_or(anyhow!("Known obstruction contains more than 32 vertices"))?
            .to_canonical();
        known_obstructions.insert(graph);
        obstructions.insert(graph);
        bar.inc(1);
    }
    bar.finish();

    /*
    eprintln!("Finding split-delete minimal obstructions");

    let bar = ProgressBar::new(obstructions.len() as u64);
    bar.set_style(progress_style.clone());

    let mut splitdel_minimal = Stats::new();

    for obs in &obstructions {
        if f4m::splitdel::find_splitdel_min(obs).is_none() {
            splitdel_minimal.add_graph(obs.clone());
        }
        bar.inc(1);
    }
    bar.finish();
    eprintln!("\n");

    splitdel_minimal.print("Split-delete minimal obstructions");
    */

    let mut new_obstructions = Stats::new();
    let mut visited = HashSet::new();

    eprintln!("Generating split-delete obstructions");

    let bar = ProgressBar::new(obstructions.len() as u64);
    bar.set_style(progress_style.clone());

    while let Some(obs) = obstructions.iter().next().cloned() {
        obstructions.remove(&obs);
        visited.insert(obs.clone());
        for new in f4m::splitdel::gen_splitdel(&obs) {
            let new = new.to_canonical();
            new_obstructions.add_graph(new);
            /*
            if !visited.contains(&new) && !obstructions.contains(&new) {
                bar.inc_length(1);
                obstructions.insert(new);
            }
            */
        }
        bar.inc(1);
    }
    bar.finish();

    eprintln!("\n");

    new_obstructions.print("Found obstructions");

    if let Some(path) = opt.unknown_file {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write unknowns to at {:?}", path))?;
        for obstruction in &new_obstructions.graphs {
            writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                .context("Cannot write new unknown obstruction")?;
        }
        file.flush()?;
    }

    Ok(())
}
