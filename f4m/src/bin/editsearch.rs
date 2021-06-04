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

#[derive(StructOpt, Debug)]
#[structopt(name = "f4m-editsearch", about = "Tool to search for toroidal obstructions with add-delete")]
struct Opt {
    #[structopt(long)]
    dist: usize,
    #[structopt(long, parse(from_os_str))]
    found_file: Option<PathBuf>,
    #[structopt(long, parse(from_os_str))]
    unknown_file: Option<PathBuf>,
    /// Other known obstructions
    #[structopt(short, long, parse(from_os_str))]
    known: Option<PathBuf>,
    /// If other known obstructions are in upper triangle format
    #[structopt(long)]
    known_triangle: bool,
    /// If graphs are in upper triangle format
    #[structopt(long)]
    triangle: bool,
    /// Obstrutions for starting point
    #[structopt(parse(from_os_str))]
    obstructions: PathBuf,
    modres: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let progress_style = ProgressStyle::default_bar()
        .template("{prefix}{bar:40.cyan/blue} {pos:>7}/{len:7} {elapsed} elapsed, est {eta} left {msg}")
        .progress_chars("#> ");

    let mut known_obstructions = HashSet::new();
    let mut search_obstructions = HashSet::new();
    
    if let Some(known) = opt.known.as_ref() {
        eprintln!("Loading other known obstructions");
        let file = std::fs::read_to_string(known)
            .with_context(|| format!("Failed to read obstructions from {:?}", known))?;
        let num = file.lines().count();
        let bar = ProgressBar::new(num as u64);
        bar.set_style(progress_style.clone());
        for (i, line) in file.lines().enumerate() {
            let mut graph = if opt.known_triangle {
                sgtk::parse::from_upper_tri::<Graph32>(line)
                    .ok_or(anyhow!("Known obstruction contains more than 32 vertices"))?
            } else {
                sgtk::parse::from_graph6::<Graph32>(line)
            }.to_canonical();
            known_obstructions.insert(graph);
            bar.inc(1);
        }
        bar.finish();
    }

    eprintln!("Loading obstructions");
    let file = std::fs::read_to_string(&opt.obstructions)
        .with_context(|| format!("Failed to read obstructions from {:?}", &opt.obstructions))?;
    let num = file.lines().count();

    let (mr_mod, mr_res) = if let Some((a, b)) = opt.modres.as_ref()
        .and_then(|s| s.split_once('/'))
    {
        let r = a.parse::<usize>()
            .with_context(|| "Unable to parse res part of mod/res")?;
        let m = b.parse::<usize>()
            .with_context(|| "Unable to parse mod part of mod/res")?;
        (m, r)
    } else {
        (1, 0)
    };

    let bar = ProgressBar::new(num as u64);
    bar.set_style(progress_style.clone());
    for (i, line) in file.lines().enumerate() {
        let mut graph = if opt.triangle {
            sgtk::parse::from_upper_tri::<Graph32>(line)
                .ok_or(anyhow!("Known obstruction contains more than 32 vertices"))?
        } else {
            sgtk::parse::from_graph6::<Graph32>(line)
        }.to_canonical();
        known_obstructions.insert(graph);
        if i % mr_mod == mr_res {
            search_obstructions.insert(graph);
        }
        bar.inc(1);
    }
    bar.finish();

    let mut found_graphs = Stats::new();
    let mut found_obs = Stats::new();
    let mut found_unknown = Stats::new();

    eprintln!("Searching for new graphs");
    let bar = ProgressBar::new(search_obstructions.len() as u64);
    bar.set_style(progress_style.clone());
    let mut searcher = f4m::editsearch::EditSearcher::new(opt.dist);
    for graph in &search_obstructions {
        searcher.search(graph.clone(), 0);
        bar.inc(1);
    }
    bar.finish();


    eprintln!("Searching for new obstructions");
    let bar = ProgressBar::new(searcher.visited.len() as u64);
    bar.set_style(progress_style.clone());

    let mut obs_searcher = f4m::splitdel::ObstructionSearcher::new();

    for graph in searcher.visited {
        obs_searcher.search(graph.clone());
        found_graphs.add_graph(graph.clone());
        bar.inc(1);
    }
    bar.finish();

    for obs in obs_searcher.found {
        found_obs.add_graph(obs.clone());
        if !known_obstructions.contains(&obs) {
            dbg!(sgtk::parse::to_graph6(&obs));
            found_unknown.add_graph(obs);
        }
    }

    found_graphs.print("Found graphs");
    found_obs.print("Found obstructions");
    found_unknown.print("Found unknown obstructions");

    if let Some(path) = opt.unknown_file {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write unknowns to at {:?}", path))?;
        for obstruction in &found_unknown.graphs {
            writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                .context("Cannot write new unknown obstruction")?;
        }
        file.flush()?;
    }

    if let Some(path) = opt.found_file {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write unknowns to at {:?}", path))?;
        for obstruction in &found_obs.graphs {
            writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                .context("Cannot write new unknown obstruction")?;
        }
        file.flush()?;
    }

    Ok(())
}
