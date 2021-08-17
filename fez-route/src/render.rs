use crate::common::heuristic_path;
use crate::rooms::Node;
use itertools::Itertools;
use log::*;
use petgraph::stable_graph::{EdgeReference, NodeIndex, StableGraph};
use petgraph::visit::{
    DfsPostOrder, EdgeRef, GraphBase, GraphRef, IntoEdgeReferences, IntoEdges, IntoEdgesDirected,
    IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences, NodeRef,
    VisitMap, Visitable, Walker,
};
use petgraph::EdgeDirection::{Incoming, Outgoing};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const EXT: &'static str = "png";

const RBG_COLOR_SCALE: &[ColorF] = &[
    (1.0, 0.0, 0.0),
    (1.0, 0.0, 1.0),
    (0.0, 0.0, 1.0),
    (0.0, 1.0, 1.0),
    (0.0, 1.0, 0.0),
];

fn color_scale(value: f64) -> ColorF {
    if value <= 0.0 {
        RBG_COLOR_SCALE[0]
    } else {
        let value = value * RBG_COLOR_SCALE.len() as f64;
        let i = value.floor() as usize;
        let f = value.fract();
        if i >= RBG_COLOR_SCALE.len() - 1 {
            RBG_COLOR_SCALE[RBG_COLOR_SCALE.len() - 1]
        } else {
            interp(RBG_COLOR_SCALE[i], RBG_COLOR_SCALE[i + 1], f)
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

    pub fn render(
        &self,
        filename: String,
        values: &StableGraph<&Node, f64>,
        first: NodeIndex,
        last: NodeIndex,
    ) {
        let heuristic: HashSet<_> = heuristic_path(values, first, last)
            .into_iter()
            .map(|e| e.id())
            .collect();

        let graph = values.filter_map(
            |i, &n| {
                if IntoIterator::into_iter([
                    values.edges_directed(i, Outgoing),
                    values.edges_directed(i, Incoming),
                ])
                .any(|mut iter| iter.next().is_some())
                {
                    Some(n.name.as_str())
                } else {
                    None
                }
            },
            |i, &e| Some((color(e), heuristic.contains(&i))),
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

fn try_render(path: &Path, graph: &StableGraph<&str, (ColorU, bool)>) -> io::Result<()> {
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
        .sorted_by_key(|n| *n.weight())
        .group_by(|n| n.weight().split('.').next().unwrap())
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
        .try_for_each(|(s, t, ((r, g, b), h))| {
            let w = if h { "3" } else { "1" };
            writeln!(
                output,
                "  \"{}\" -> \"{}\" [ color = \"#{:02x}{:02x}{:02x}\" penwidth = {}];",
                s, t, r, g, b, w
            )
        })?;

    writeln!(output, "}}")?;

    child.wait()?;
    Ok(())
}
