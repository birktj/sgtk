use sgtk::graph::Graph64;
use sgtk::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::{anyhow, Context, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "show-graphs", about = "Tool to render graphs in graph6 format.")]
struct Opt {
    /// Output file
    #[structopt(short, long)]
    output: String,
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let file = std::fs::read_to_string(&opt.input)
        .with_context(|| format!("Failed to read known obstrucions from {:?}", &opt.input))?;

    let graphs: Vec<Graph64> = file.lines()
        .map(|l| sgtk::parse::from_graph6(l))
        .collect();

    sgtk::viz::render_dot(&opt.output, &graphs);
    Ok(())
}
