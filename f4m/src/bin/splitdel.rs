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
#[structopt(name = "f4m-splitdel", about = "Tool to search for toroidal obstructions with split-delete.")]
struct Opt {
    /// File to write the split-delete graph to
    #[structopt(long, parse(from_os_str))]
    splitdel_graph: Option<PathBuf>,
    /// File to write unkown obstructions to
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

    /*
    eprintln!("Finding split-delete minimal obstructions");

    let bar = ProgressBar::new(search_obstructions.len() as u64);
    bar.set_style(progress_style.clone());

    let mut splitdel_minimal = Stats::new();

    for obs in &search_obstructions {
        if f4m::splitdel::find_splitdel_min(obs).is_none() {
            splitdel_minimal.add_graph(obs.clone());
        }
        bar.inc(1);
    }
    bar.finish();
    eprintln!("\n");

    splitdel_minimal.print("Split-delete minimal obstructions");
    */

    let mut splitdel_graph = HashMap::new();

    let mut found_obstructions = Stats::new();
    let mut new_obstructions = Stats::new();

    eprintln!("Generating split-delete obstructions");

    let bar = ProgressBar::new(search_obstructions.len() as u64);
    bar.set_style(progress_style.clone());

    while let Some(obs) = search_obstructions.iter().next().cloned() {
        search_obstructions.remove(&obs);
        let mut splitdel_line = HashSet::new();
        for new in f4m::splitdel::gen_splitdel(&obs) {
            let new = new.to_canonical();
            splitdel_line.insert(new.clone());
            found_obstructions.add_graph(new);
            if !search_obstructions.contains(&new) && !known_obstructions.contains(&new) {
                bar.inc_length(1);
                search_obstructions.insert(new);
            }
            if !known_obstructions.contains(&new) {
                new_obstructions.add_graph(new);
                known_obstructions.insert(new);
            }
        }
        splitdel_graph.insert(obs.clone(), splitdel_line);
        bar.inc(1);
    }
    bar.finish();

    eprintln!("\n");

    found_obstructions.print("Found obstructions");
    new_obstructions.print("New obstructions");

    if let Some(path) = opt.unknown_file {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write unknowns to at {:?}", path))?;
        for obstruction in &new_obstructions.graphs {
            writeln!(file, "{}", sgtk::parse::to_graph6(obstruction))
                .context("Cannot write new unknown obstruction")?;
        }
        file.flush()?;
    }

    if let Some(path) = opt.splitdel_graph {
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Cannot create file to write split-delete graph to at {:?}", path))?;
        for (obs, line) in &splitdel_graph {
            write!(file, "{}", sgtk::parse::to_graph6(obs))
                .context("Cannot write obstruction to the split-delete graph file")?;
            for graph in line {
                write!(file, " {}", sgtk::parse::to_graph6(graph))
                    .context("Cannot write obstruction to the split-delete graph file")?;
            }
            writeln!(file, "").context("Cannot write newline to the split-delete graph file")?;
        }
        file.flush()?;
    }


    Ok(())
}
