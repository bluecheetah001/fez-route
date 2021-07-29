use glpk::*;
use itertools::Itertools;
use log::*;
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use petgraph::visit::{
    Dfs, EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeRef, VisitMap,
};
use petgraph::Direction::{Incoming, Outgoing};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const EPS: f64 = 1e-6;

#[derive(Debug)]
pub struct Edge {
    pub frames: i32,
}
#[derive(Debug)]
pub struct Node {
    // globally unique name, should start with {room}-
    pub name: String,
    // bits gained in this node
    pub bits: i32,
    // keys gained in this node
    pub keys: i32,
    // handles water level and the two indirect secret doors
    // use NodeIndex::end() for no constraint
    pub after_node: NodeIndex,
    // for the big doors
    // pub required_bits: i32,

    // TODO something to handle sewer water levels? can that work in a similar way to after_node?
    // probably still a different field though
}
impl Default for Node {
    fn default() -> Self {
        Self {
            name: String::new(),
            bits: 0,
            keys: 0,
            after_node: NodeIndex::end(),
        }
    }
}

pub fn optimize(graph: &Graph<Node, Edge>, required_bits: i32) {
    let first_node = graph
        .externals(Incoming)
        .exactly_one()
        .ok()
        .expect("exactly one source node");
    let last_node = graph
        .externals(Outgoing)
        .exactly_one()
        .ok()
        .expect("exactly one target node");

    let mut problem = Problem::new();
    problem.set_name("FEZ any% route".to_owned());
    problem.set_direction(Direction::Minimize);

    // vars
    // if an edge should be taken
    let edges = problem.add_vars(
        graph
            .edge_references()
            .map(|e| {
                let edge = e.weight();
                let source = &graph[e.source()];
                let target = &graph[e.target()];
                Var {
                    name: format!("{}.to.{}", source.name, target.name),
                    kind: Kind::Int,
                    bounds: Bounds::Double(0.0, 1.0),
                    objective: edge.frames as f64,
                }
            })
            .collect(),
    );

    // exprs
    // node input/output edge conditions
    problem.add_exprs(
        graph
            .node_references()
            .map(|(n, node)| Expr {
                name: format!("{}.flow", node.name),
                bounds: Bounds::Fixed(if n == first_node {
                    1.0
                } else if n == last_node {
                    -1.0
                } else {
                    0.0
                }),
                terms: graph
                    .edges_directed(n, Incoming)
                    .map(|e| edges.get(e.id().index()) * -1.0)
                    .chain(
                        graph
                            .edges_directed(n, Outgoing)
                            .map(|e| edges.get(e.id().index()) * 1.0),
                    )
                    .collect(),
            })
            .collect(),
    );
    problem.add_exprs(
        graph
            .node_references()
            .filter(|&(n, _)| n != first_node && n != last_node)
            .map(|(n, node)| Expr {
                name: format!("{}.once", node.name),
                bounds: Bounds::Upper(1.0),
                terms: graph
                    .edges_directed(n, Outgoing)
                    .map(|e| edges.get(e.id().index()) * 1.0)
                    .collect(),
            })
            .collect(),
    );

    // get enough bits
    // not bothering with other required_bits nodes yet since they shouldn't be violated based on timing data
    problem.add_expr(Expr {
        name: "total_bits".to_owned(),
        bounds: Bounds::Lower(required_bits as f64),
        terms: graph
            .node_references()
            .filter(|&(_, node)| node.bits != 0)
            .flat_map(|(n, node)| {
                graph
                    .edges_directed(n, Incoming)
                    .map(move |e| edges.get(e.id().index()) * node.bits as f64)
            })
            .collect(),
    });
    // get enough keys
    // not sure yet if order should be enforced by cuts or potential vars
    // probably potential vars since cuts don't suggest a direction
    problem.add_expr(Expr {
        name: "total_keys".to_owned(),
        bounds: Bounds::Lower(0.0),
        terms: graph
            .node_references()
            .filter(|&(_, node)| node.keys != 0)
            .flat_map(|(n, node)| {
                graph
                    .edges_directed(n, Incoming)
                    .map(move |e| edges.get(e.id().index()) * node.keys as f64)
            })
            .collect(),
    });
    // after node condition
    // not sure yet if order should be enforced by cuts or potential vars
    // probably potential vars since cuts don't suggest a direction
    problem.add_exprs(
        graph
            .node_references()
            .filter(|(_, node)| node.after_node != NodeIndex::end())
            .map(|(n, node)| Expr {
                name: format!("{}.after.{}", node.name, graph[node.after_node].name),
                bounds: Bounds::Lower(0.0),
                terms: graph
                    .edges_directed(node.after_node, Incoming)
                    .map(|e| edges.get(e.id().index()) * 1.0)
                    .chain(
                        graph
                            .edges_directed(n, Incoming)
                            .map(|e| edges.get(e.id().index()) * -1.0),
                    )
                    .collect(),
            })
            .collect(),
    );

    info!("built problem");

    let mut renderer = Renderer::new("rendered").unwrap();
    // let mut index = 0;
    // let mut output = graph.map(|_, n| n, |_, _| 0.0);
    // let mut render = |problem: &Prob, state| {
    //     if state == "done" {
    //         graph.edge_references().for_each(|e| {
    //             output[e.id()] = problem.get_int_value(edges.get(e.id().index()));
    //         });
    //     } else {
    //         graph.edge_references().for_each(|e| {
    //             output[e.id()] = problem.get_value(edges.get(e.id().index()));
    //         });
    //     }
    //     index += 1;
    //     let path = format!("rendered/{:03}-{}.dot", index, state);
    //     render(Path::new(&path), &graph, problem);
    // };

    problem
        .optimize_mip(|reason| match reason {
            Reason::AddLazyExprs(problem) => {
                // graph.edge_references().for_each(|e| {
                //     let value = problem.get_value(edges.get(e.id().index()));
                //     if value >= 1e-6 {
                //         trace!(
                //             "edge {:02}->{:02} v:{}",
                //             e.source().index(),
                //             e.target().index(),
                //             value
                //         );
                //     }
                // });
                let connected = EdgeFiltered::from_fn(graph, |e| {
                    problem.get_value(edges.get(e.id().index())) >= EPS
                });

                let mut bits = 0;
                let mut dfs = Dfs::new(&connected, first_node);
                while let Some(n) = dfs.next(&connected) {
                    bits += graph[n].bits;
                    if bits >= required_bits {
                        break;
                    }
                }
                if bits < required_bits {
                    trace!("adding cut {:?}", dfs.discovered);
                    renderer.before_cut(&graph, &edges, &problem);
                    problem.add_expr(Expr {
                        name: format!("cut.{:?}", dfs.discovered),
                        bounds: Bounds::Lower(1.0),
                        terms: dfs
                            .discovered
                            .ones()
                            .map(NodeIndex::new)
                            .flat_map(|n| {
                                graph
                                    .edges_directed(n, Outgoing)
                                    .filter(|e| !dfs.discovered.is_visited(&e.target()))
                                    .map(|e| edges.get(e.id().index()) * 1.0)
                            })
                            .collect(),
                    });
                } else {
                    trace!("solved relaxation");
                    renderer.before_branch(&graph, &edges, &problem);
                }
            }
            Reason::NewBestSolution(problem) => {
                renderer.new_solution(&graph, &edges, &problem);
                info!("new best solution");
            }
        })
        .unwrap();
    renderer.best_solution(&graph, &edges, &problem);
    trace!("done!");
}

struct Renderer {
    folder: PathBuf,
    index: usize,
}
impl Renderer {
    fn new(folder: impl Into<PathBuf>) -> Option<Renderer> {
        let folder = folder.into();
        if let Err(e) = Renderer::try_init(&folder) {
            error!("failed to setup rendering into {:?}: {}", folder, e);
            return None;
        }
        Some(Renderer { folder, index: 0 })
    }
    fn try_init(folder: &Path) -> io::Result<()> {
        fs::remove_dir_all(folder)?;
        fs::create_dir_all(folder)?;
        Ok(())
    }

    fn before_cut(
        &mut self,
        graph: &Graph<Node, Edge>,
        edges: &glpk::VarRefs,
        problem: &glpk::Prob,
    ) {
        self.index += 1;
        let path = self.folder.join(format!("{:03}-cut.dot", self.index));
        render(&path, graph, edges, problem, false);
    }

    fn before_branch(
        &mut self,
        graph: &Graph<Node, Edge>,
        edges: &glpk::VarRefs,
        problem: &glpk::Prob,
    ) {
        self.index += 1;
        let path = self.folder.join(format!("{:03}-branch.dot", self.index));
        render(&path, graph, edges, problem, false);
    }

    fn new_solution(
        &mut self,
        graph: &Graph<Node, Edge>,
        edges: &glpk::VarRefs,
        problem: &glpk::Prob,
    ) {
        self.index += 1;
        let path = self.folder.join(format!("{:03}-solved.dot", self.index));
        render(&path, graph, edges, problem, false);
    }

    fn best_solution(
        &mut self,
        graph: &Graph<Node, Edge>,
        edges: &glpk::VarRefs,
        problem: &glpk::Prob,
    ) {
        self.index += 1;
        let path = self.folder.join(format!("{:03}-best.dot", self.index));
        render(&path, graph, edges, problem, true);
    }
}

pub fn render(
    path: &Path,
    graph: &Graph<Node, Edge>,
    edges: &glpk::VarRefs,
    problem: &glpk::Prob,
    is_int: bool,
) {
    if let Err(e) = try_render(path, graph, edges, problem, is_int) {
        error!("failed to generate graphviz at {:?}: {}", path, e);
    }
}

fn try_render(
    path: &Path,
    graph: &Graph<Node, Edge>,
    edges: &glpk::VarRefs,
    problem: &glpk::Prob,
    is_int: bool,
) -> io::Result<()> {
    // let mut child = Command::new("fdp")
    //     .arg("-T")
    //     .arg(path.extension().expect("no extension"))
    //     .arg("-o")
    //     .arg(path)
    //     .stdin(Stdio::piped())
    //     .spawn()?;
    // let mut output = child.stdin.as_ref().unwrap();
    let mut output = fs::File::create(path)?;

    writeln!(output, "strict digraph {{")?;

    graph
        .node_references()
        .sorted_by(|l, r| l.1.name.cmp(&r.1.name))
        .group_by(|n| n.1.name.split('-').next().unwrap())
        .into_iter()
        .try_for_each(|(k, mut g)| {
            writeln!(output, "  subgraph \"cluster-{}\" {{", k)?;
            writeln!(output, "    label = \"{}\"", k)?;
            g.try_for_each(|n| {
                writeln!(
                    output,
                    "    \"{}\" [ label = \"{}\" ];",
                    n.1.name,
                    &n.1.name[(k.len() + 1)..]
                )
            })?;
            writeln!(output, "  }}")
        })?;

    graph
        .edge_references()
        .map(|e| {
            (
                &graph[e.source()].name,
                &graph[e.target()].name,
                if is_int {
                    problem.get_int_value(edges.get(e.id().index()))
                } else {
                    problem.get_value(edges.get(e.id().index()))
                },
            )
        })
        // .sorted_by(|l, r| l.0.cmp(r.0).then(l.1.cmp(r.1)))
        .try_for_each(|(s, t, w)| {
            let color = if w >= EPS {
                (w.min(1.0) * 127.0 + 128.0) as u8
            } else {
                16
            };
            writeln!(
                output,
                "  \"{}\" -> \"{}\" [ color = \"#000000{:02x}\" ];",
                s, t, color
            )
        })?;

    writeln!(output, "}}")?;

    // child.wait()?;
    Ok(())
}
