use std::process::{Command, Stdio};
use crate::prelude::*;

pub struct GraphvizOptions {
    shape: String,
    colorscheme: String,
}

//pub fn graph2dot<G: Graph>(graphs: &[(G, Option<Coloring16>)]) -> String {

pub fn graph2tikz<G: Graph>(graph: &G) -> String {
    use std::fmt::Write;
    let mut tikz = String::new();
    write!(tikz, "\\tikz \\graph[spring electrical layout]{{\n").unwrap();
    for u in graph.nodes().iter() {
        write!(tikz, "{};\n", u).unwrap();
    }
    for (u, v) in graph.edges() {
        write!(tikz, "{}--{};\n", u, v).unwrap();
    }
    write!(tikz, "}};\n").unwrap();
    tikz
}

pub fn graph2dot<G: Graph>(graphs: &[G]) -> String {
    use std::fmt::Write;
    let mut dot = String::new();

    let opts = GraphvizOptions {
        shape: "circle".to_string(),
        colorscheme: "set312".to_string(),
    };


    write!(dot, "graph {{\n").unwrap();
    write!(dot, "    node[shape = {} width=0.2 style=filled colorscheme={}]\n", 
        opts.shape, opts.colorscheme).unwrap();
    for (gi, graph) in graphs.iter().enumerate() {
        write!(dot, "subgraph cluster{} {{\n", gi).unwrap();
        write!(dot, "    label=\"{}\";\n", gi).unwrap();
        for u in graph.nodes().iter() {
            /*
            if let Some(c) = coloring.as_ref()
                .map(|coloring| coloring.get(u))
            {
                write!(dot, "    g{}n{}[label=\"{}\", fillcolor={}];\n", gi, u, u, c+1)
                    .unwrap();
            } else {
            */
                write!(dot, "    g{}n{}[label=\"{}\"];\n", gi, u, u)
                    .unwrap();
            //}
            //write!(dot, "    {};\n", u);
        }
        for (u, v) in graph.edges() {
            write!(dot, "    g{}n{} -- g{}n{};\n", gi, u, gi, v)
                .unwrap();
        }
        write!(dot, "}}\n").unwrap();
    }
    write!(dot, "}}\n").unwrap();
    dot
}

//pub fn render_dot(file: &str, graphs: &[(Graph16, Option<Coloring16>)]) {
pub fn render_dot<G: Graph>(file: &str, graphs: &[G]) {
    use std::io::Write;
    let dot = graph2dot(graphs);

    let mut proc = Command::new("fdp")
        .arg("-Tpdf")
        .arg("-o").arg(file)
        .stdin(Stdio::piped())
        .spawn().unwrap();

    //dbg!(&dot);

    {
        let mut stdin = proc.stdin.take().unwrap();
        write!(stdin, "{}", dot).unwrap();
        stdin.flush().unwrap();
    }
    proc.wait().unwrap();
}
