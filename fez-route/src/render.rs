use crate::rooms::Node;
use itertools::Itertools;
use log::*;
use petgraph::graph::Graph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::EdgeDirection::{Incoming, Outgoing};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const EXT: &'static str = "png";
const EPS: f64 = 1e-6;

const COLOR_SCALE: [ColorF; 5] = [
    (1.0, 0.0, 0.0),
    (1.0, 0.0, 1.0),
    (0.0, 0.0, 1.0),
    (0.0, 1.0, 1.0),
    (0.0, 1.0, 0.0),
];
fn color_scale(value: f64) -> ColorF {
    if value <= 0.0 {
        COLOR_SCALE[0]
    } else {
        let value = value * COLOR_SCALE.len() as f64;
        let i = value.floor() as usize;
        let f = value.fract();
        if i >= COLOR_SCALE.len() - 1 {
            COLOR_SCALE[COLOR_SCALE.len() - 1]
        } else {
            interp(COLOR_SCALE[i], COLOR_SCALE[i + 1], f)
        }
    }
}

fn interp(a: ColorF, b: ColorF, f: f64) -> ColorF {
    (
        f * (b.0 - a.0) + a.0,
        f * (b.1 - a.1) + a.1,
        f * (b.2 - a.2) + a.2,
    )
}

fn scale(a: ColorF, s: f64) -> ColorF {
    (s * a.0, s * a.1, s * a.2)
}

fn as_bytes(a: ColorF) -> ColorU {
    (as_byte(a.0), as_byte(a.1), as_byte(a.2))
}

fn as_byte(v: f64) -> u8 {
    (v * 255.0).ceil() as u8
}

pub struct Renderer {
    folder: PathBuf,
}
impl Renderer {
    pub fn new(folder: impl Into<PathBuf>) -> Option<Self> {
        let folder = folder.into();
        if let Err(e) = Renderer::try_init(&folder) {
            error!("failed to setup rendering into {:?}: {}", folder, e);
            return None;
        }
        Some(Renderer { folder })
    }

    fn try_init(folder: &Path) -> io::Result<()> {
        match fs::remove_dir_all(folder) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            e => return e,
        };
        fs::create_dir_all(folder)?;
        Ok(())
    }

    pub fn render(&self, filename: String, graph: &Graph<&Node, f64>) {
        let graph = graph.filter_map(
            |i, &n| {
                if IntoIterator::into_iter([
                    graph.edges_directed(i, Outgoing),
                    graph.edges_directed(i, Incoming),
                ])
                .flatten()
                .any(|e| *e.weight() > EPS)
                {
                    Some(n.name.as_str())
                } else {
                    None
                }
            },
            |_, &e| {
                if e > EPS {
                    Some(color(e))
                } else {
                    None
                }
            },
        );

        let path = self.folder.join(filename);
        if let Err(e) = try_render(&path, &graph) {
            error!("failed to generate graphviz at {:?}: {}", path, e);
        }
    }

    pub fn render_diff(
        &self,
        filename: String,
        prev: &Graph<&Node, f64>,
        next: &Graph<&Node, f64>,
    ) {
        self.render(filename, next);
        return;

        let graph = next.filter_map(
            |i, &n| {
                if IntoIterator::into_iter([
                    prev.edges_directed(i, Outgoing),
                    prev.edges_directed(i, Incoming),
                    next.edges_directed(i, Outgoing),
                    next.edges_directed(i, Incoming),
                ])
                .flatten()
                .any(|e| *e.weight() > EPS)
                {
                    Some(n.name.as_str())
                } else {
                    None
                }
            },
            |i, &next_e| {
                let prev_e = prev.edge_weight(i).map_or(0.0, |&prev_e| prev_e);
                if next_e > EPS || prev_e > EPS {
                    Some(diff_color(prev_e, next_e))
                } else {
                    None
                }
            },
        );

        let path = self.folder.join(filename);
        if let Err(e) = try_render(&path, &graph) {
            error!("failed to generate graphviz at {:?}: {}", path, e);
        }
    }
}

type ColorF = (f64, f64, f64);
type ColorU = (u8, u8, u8);

fn color(value: f64) -> ColorU {
    as_bytes(color_scale(value))
}

fn diff_color(prev: f64, next: f64) -> ColorU {
    let dist = prev.max(next);
    let angle = if next < prev {
        next / prev / 2.0
    } else if next == prev {
        0.5
    } else {
        1.0 - prev / next / 2.0
    };

    let color = color_scale(angle);

    as_bytes(scale(color, dist * 0.8 + 0.2))
}

fn try_render(path: &Path, graph: &Graph<&str, ColorU>) -> io::Result<()> {
    let mut child = Command::new("fdp")
        .arg("-T")
        .arg(EXT)
        .arg("-o")
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()?;
    let mut output = child.stdin.as_ref().unwrap();
    // let mut output = fs::File::create(path)?;

    // TODO black background with white lines
    writeln!(output, "strict digraph {{")?;
    writeln!(
        output,
        "  graph [ bgcolor = \"black\" color = \"white\" fontcolor = \"white\" ]"
    )?;
    writeln!(output, "  node [ color = \"white\" fontcolor = \"white\" ]")?;
    writeln!(output, "  edge [ penwidth = 2 ]")?;

    graph
        .node_references()
        .sorted_by_key(|(_, &n)| n)
        .group_by(|(_, &n)| n.split('.').next().unwrap())
        .into_iter()
        .try_for_each(|(k, mut g)| {
            writeln!(output, "  subgraph \"cluster-{}\" {{", k)?;
            writeln!(output, "    label = \"{}\"", k)?;
            g.try_for_each(|(_, &n)| {
                writeln!(
                    output,
                    "    \"{}\" [ label = \"{}\" ];",
                    n,
                    &n[(k.len() + 1)..]
                )
            })?;
            writeln!(output, "  }}")
        })?;

    graph
        .edge_references()
        .map(|e| (graph[e.source()], graph[e.target()], *e.weight()))
        .try_for_each(|(s, t, (r, g, b))| {
            writeln!(
                output,
                "  \"{}\" -> \"{}\" [ color = \"#{:02x}{:02x}{:02x}\" ];",
                s, t, r, g, b
            )
        })?;

    writeln!(output, "}}")?;

    child.wait()?;
    Ok(())
}
