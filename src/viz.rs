use std::process::{Command, Stdio};
use crate::Graph16;
use crate::graph::Coloring16;


pub struct GraphvizOptions {
    shape: String,
    colorscheme: String,
}

pub fn graph2dot(graphs: &[(Graph16, Option<Coloring16>)]) -> String {
    use std::fmt::Write;
    let mut dot = String::new();

    let opts = GraphvizOptions {
        shape: "circle".to_string(),
        colorscheme: "set312".to_string(),
    };


    write!(dot, "graph {{\n");
    write!(dot, "    node[shape = {} width=0.2 style=filled colorscheme={}]\n", 
        opts.shape, opts.colorscheme);
    for (gi, (graph, coloring)) in graphs.iter().enumerate() {
        write!(dot, "subgraph cluster{} {{\n", gi);
        write!(dot, "    label=\"{}\";\n", gi);
        for u in graph.nodes() {
            if let Some(c) = coloring.as_ref()
                .map(|coloring| coloring.get(u))
            {
                write!(dot, "    g{}n{}[label=\"{}\", fillcolor={}];\n", gi, u, u, c+1);
            } else {
                write!(dot, "    g{}n{}[label=\"{}\"];\n", gi, u, u);
            }
            //write!(dot, "    {};\n", u);
        }
        for (u, v) in graph.edges() {
            write!(dot, "    g{}n{} -- g{}n{};\n", gi, u, gi, v);
        }
        write!(dot, "}}\n");
    }
    write!(dot, "}}\n");
    dot
}

pub fn render_dot(file: &str, graphs: &[(Graph16, Option<Coloring16>)]) {
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
