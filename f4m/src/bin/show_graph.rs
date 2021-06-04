use sgtk::graph::Graph64;
use sgtk::prelude::*;
use std::path::PathBuf;
use std::io::Write;
use structopt::StructOpt;
use anyhow::{anyhow, Context, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "show-graphs", about = "Tool to render graphs in graph6 format.")]
struct Opt {
    /// Tikz format
    #[structopt(long)]
    tikz: bool,
    /// Input is in upper triangle format
    #[structopt(long)]
    triangle: bool,
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

    let graphs: Vec<Graph64> = if opt.triangle {
        file.lines()
            .map(|l| sgtk::parse::from_upper_tri(l)
                .ok_or(anyhow!("Could not parse graph")))
            .collect::<Result<_>>()?
    } else {
        file.lines()
            .map(|l| sgtk::parse::from_graph6(l))
            .collect()
    };

    if opt.tikz {
        let mut output = std::fs::File::create(opt.output)?;
        for graph in graphs {
            let tikz = sgtk::viz::graph2tikz(&graph);
            write!(output, "{}\n", tikz)
                .with_context(|| "Cannot write tikz graph to file")?;
        }
    } else {
        sgtk::viz::render_dot(&opt.output, &graphs);
    }
    Ok(())
}
