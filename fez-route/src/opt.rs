use crate::render::{Renderer, EXT};
use crate::rooms::{Cost, Edge, Node};
use fixedbitset::FixedBitSet;
use glpk::*;
use itertools::Itertools;
use log::*;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::{Dfs, EdgeFiltered, EdgeRef, IntoNodeReferences};
use petgraph::Direction::{Incoming, Outgoing};

const EPS: f64 = 1e-6;

impl Edge {
    fn get_frames(&self) -> f64 {
        let dist = self.time;
        let xz_frames = dist.dx.min(dist.dz) * 15.0;
        let y_frames = dist.dy * 30.0;
        xz_frames.max(y_frames)
    }
}
impl Node {
    fn keys_minus_lock(&self) -> i32 {
        self.keys
            + match self.cost {
                Some(Cost::Lock) => -1,
                _ => 0,
            }
    }
}

pub fn optimize(graph: &Graph<Node, Edge>, required_bits: i32) {
    graph.externals(Incoming).for_each(|node| {
        info!("incoming: {}", graph[node].name);
    });
    graph.externals(Outgoing).for_each(|node| {
        info!("outgoing: {}", graph[node].name);
    });

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
    let edges = problem.add_vars(edge_vars(graph));

    // exprs
    problem.add_exprs(flow_exprs(graph, edges, first_node, last_node));
    problem.add_exprs(capacity_exprs(graph, edges, first_node, last_node));
    problem.add_expr(required_bits_expr(graph, edges, required_bits));
    problem.add_expr(total_keys_expr(graph, edges));
    // problem.add_exprs(approx_water_lock_exprs(graph, edges));

    info!("built problem");

    let mut index = 0;
    let mut prev = Graph::new();
    let mut renderer = Renderer::new("rendered").unwrap();

    problem
        .optimize_mip(|reason| match reason {
            Reason::AddLazyExprs(problem) => {
                let value_graph =
                    graph.map(|_, n| n, |i, _| problem.get_value(edges.get(i.index())));
                if let Some(expr) =
                    lazy_required_bits_expr(&value_graph, edges, first_node, required_bits)
                {
                    trace!("adding cut {}: {}", index, expr.name);
                    renderer.render_diff(format!("{}-cut.{}", index, EXT), &prev, &value_graph);
                    index += 1;
                    prev = value_graph;
                    problem.add_expr(expr);
                } else {
                    trace!("solved relaxation {}", index);
                    renderer.render_diff(format!("{}-branch.{}", index, EXT), &prev, &value_graph);
                    index += 1;
                    prev = value_graph;
                    // abort();
                }
            }
            Reason::NewBestSolution(problem) => {
                let value_graph =
                    graph.map(|_, n| n, |i, _| problem.get_value(edges.get(i.index())));
                info!("new best solution {}", index);
                renderer.render(format!("{}-solved.{}", index, EXT), &value_graph);
                index += 1;
            }
        })
        .unwrap();

    renderer.render(
        format!("{}-best.{}", index, EXT),
        &graph.map(|_, n| n, |i, _| problem.get_int_value(edges.get(i.index()))),
    );
    trace!("done!");
}

fn edge_vars(graph: &Graph<Node, Edge>) -> Vec<Var> {
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
                objective: edge.get_frames() + target.time,
            }
        })
        .collect()
}

fn flow_exprs(
    graph: &Graph<Node, Edge>,
    edges: VarRefs,
    first_node: NodeIndex,
    last_node: NodeIndex,
) -> Vec<Expr> {
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
        .collect()
}

fn capacity_exprs(
    graph: &Graph<Node, Edge>,
    edges: VarRefs,
    first_node: NodeIndex,
    last_node: NodeIndex,
) -> Vec<Expr> {
    graph
        .node_references()
        .filter(|&(n, _)| n != first_node && n != last_node)
        .map(|(n, node)| Expr {
            name: format!("{}.capacity", node.name),
            bounds: Bounds::Upper(1.0),
            terms: graph
                .edges_directed(n, Outgoing)
                .map(|e| edges.get(e.id().index()) * 1.0)
                .collect(),
        })
        .collect()
}

/// not bothering with other required_bits nodes yet since they shouldn't be violated based on timing data
fn required_bits_expr(graph: &Graph<Node, Edge>, edges: VarRefs, required_bits: i32) -> Expr {
    Expr {
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
    }
}

fn total_keys_expr(graph: &Graph<Node, Edge>, edges: VarRefs) -> Expr {
    Expr {
        name: "total_keys".to_owned(),
        bounds: Bounds::Lower(0.0),
        terms: graph
            .node_references()
            .filter(|&(_, node)| node.keys_minus_lock() != 0)
            .flat_map(|(n, node)| {
                graph
                    .edges_directed(n, Incoming)
                    .map(move |e| edges.get(e.id().index()) * node.keys_minus_lock() as f64)
            })
            .collect(),
    }
}

// fn approx_water_lock_exprs(graph: &Graph<Node, Edge>, edges: VarRefs) -> Vec<Expr> {
//     graph
//         .node_references()
//         .filter(|(_, node)| node.after_node != NodeIndex::end())
//         .map(|(n, node)| Expr {
//             name: format!("{}.after.{}", node.name, graph[node.after_node].name),
//             bounds: Bounds::Lower(0.0),
//             terms: graph
//                 .edges_directed(node.after_node, Incoming)
//                 .map(|e| edges.get(e.id().index()) * 1.0)
//                 .chain(
//                     graph
//                         .edges_directed(n, Incoming)
//                         .map(|e| edges.get(e.id().index()) * -1.0),
//                 )
//                 .collect(),
//         })
//         .collect()
// }

fn lazy_required_bits_expr(
    graph: &Graph<&Node, f64>,
    edges: VarRefs,
    first_node: NodeIndex,
    required_bits: i32,
) -> Option<Expr> {
    let (connected_nodes, connected_bits) = get_connected_nodes(graph, first_node);
    let to_cut = if connected_bits < required_bits {
        Some(connected_nodes)
    // } else if is_fully_connected(graph, &connected_nodes) {
    //     // is_fully_connected isn't necessary but I think will make it faster
    //     // it will also be useful in logs for debugging
    //     None
    // } else {
    //     cut_incomplete_connected(graph, connected_nodes, connected_bits - required_bits)
    // };
    } else {
        None
    };
    to_cut.map(|to_cut| Expr {
        name: format!("cut.{:?}", to_cut),
        bounds: Bounds::Lower(1.0),
        terms: to_cut
            .ones()
            .map(NodeIndex::new)
            .flat_map(|n| {
                graph
                    .edges_directed(n, Outgoing)
                    .filter(|e| !to_cut.contains(e.target().index()))
                    .map(|e| edges.get(e.id().index()) * 1.0)
            })
            .collect(),
    })
}

fn get_connected_nodes(graph: &Graph<&Node, f64>, first_node: NodeIndex) -> (FixedBitSet, i32) {
    let mut connected_bits = 0;
    let connected = EdgeFiltered::from_fn(graph, |e| *e.weight() > EPS);
    let mut dfs = Dfs::new(&connected, first_node);
    while let Some(n) = dfs.next(&connected) {
        connected_bits += graph[n].bits;
    }
    (dfs.discovered, connected_bits)
}

fn is_fully_connected(graph: &Graph<&Node, f64>, connected_nodes: &FixedBitSet) -> bool {
    graph
        .node_indices()
        .filter(|&i| !connected_nodes.contains(i.index()))
        .all(|i| node_weight(graph, i) <= EPS)
}

fn cut_incomplete_connected(
    graph: &Graph<&Node, f64>,
    mut connected_nodes: FixedBitSet,
    mut remaining_bits: i32,
) -> Option<FixedBitSet> {
    let mut node_bit_weights = connected_nodes
        .ones()
        .map(NodeIndex::new)
        .filter(|&i| graph[i].bits > 0)
        .filter_map(|i| {
            let weight = node_weight(graph, i);
            if weight > EPS && 1.0 - weight > EPS {
                Some((i, graph[i], weight))
            } else {
                None
            }
        })
        .sorted_by(|&l, &r| {
            let l_rate = l.2 / l.1.bits as f64;
            let r_rate = r.2 / r.1.bits as f64;
            l_rate.partial_cmp(&r_rate).unwrap()
        });
    let mut remaining_weight = 1.0;
    while let Some((i, node, weight)) = node_bit_weights.next() {
        if remaining_weight - weight > EPS {
            connected_nodes.set(i.index(), false);
            remaining_bits -= node.bits;
            remaining_weight -= weight;
            if remaining_bits < 0 {
                // TODO this isn't true, it allows bypassing the cut by making a loop over the disconnected parts
                // this could be cleaned up to remove excess nodes that are no longer connected
                // but theoretically it doesn't matter since their contribution to the cut is moot
                return Some(connected_nodes);
            }
        } else if node.bits == 1 {
            return None;
        }
    }
    return None;
}

fn node_weight(graph: &Graph<&Node, f64>, i: NodeIndex) -> f64 {
    graph
        .edges_directed(i, Incoming)
        .filter(|e| *e.weight() > EPS)
        .map(|e| *e.weight())
        .sum()
}
