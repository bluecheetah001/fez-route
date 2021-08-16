use crate::render::{Renderer, EXT};
use crate::rooms::{Cost, Edge, Node};
use fixedbitset::FixedBitSet;
use glpk::*;
use itertools::{EitherOrBoth, Itertools};
use log::*;
use noisy_float::prelude::*;
use petgraph::algo::dominators;
use petgraph::stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph};
use petgraph::visit::{
    Bfs, Dfs, DfsPostOrder, EdgeFiltered, EdgeRef, GraphBase, GraphRef, IntoEdgeReferences,
    IntoEdges, IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
    IntoNodeReferences, Reversed, Topo, VisitMap, Visitable, Walker,
};
use petgraph::Direction::{Incoming, Outgoing};
use serde::__private::doc;
use std::collections::HashMap;
use std::iter;

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

/// although graph is a StableGraph, it must be initialized with fully dense node and edge indicies
pub fn optimize(graph: &StableGraph<Node, Edge>, required_bits: i32) {
    graph.externals(Incoming).for_each(|node| {
        info!("incoming: {}", graph[node].name);
    });
    graph.externals(Outgoing).for_each(|node| {
        info!("outgoing: {}", graph[node].name);
    });

    graph
        .node_indices()
        .enumerate()
        .filter(|(i, n)| *i != n.index())
        .for_each(|(i, n)| {
            panic!(
                "graph node indicies were not dense, index {} has id {}",
                i,
                n.index()
            )
        });
    graph
        .edge_indices()
        .enumerate()
        .filter(|(i, e)| *i != e.index())
        .for_each(|(i, e)| {
            panic!(
                "graph edge indicies were not dense, index {} has id {}",
                i,
                e.index()
            )
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
    // these dont' actually work that well since the 3 cycle can just go in both directions
    problem.add_exprs(no_2_cycles(graph, edges));
    // problem.add_exprs(no_3_cycles(graph, edges));
    problem.add_expr(required_bits_expr(graph, edges, required_bits));
    // problem.add_expr(total_keys_expr(graph, edges));
    // problem.add_exprs(approx_water_lock_exprs(graph, edges));

    info!("built problem");

    struct Closure<'g> {
        graph: &'g StableGraph<Node, Edge>,
        edges: VarRefs,
        first_node: NodeIndex,
        last_node: NodeIndex,
        required_bits: i32,

        index: i32,
        renderer: Renderer,
    }

    let mut closure = Closure {
        graph,
        edges,
        first_node,
        last_node,
        required_bits,

        index: 0,
        renderer: Renderer::new("rendered").unwrap(),
    };

    impl<'g> MipCallback for Closure<'g> {
        fn get_lazy_expr(&mut self, problem: &Prob) -> Option<Expr> {
            let value_graph = value_graph(self.graph, problem, self.edges);
            // TODO or small disconnected cycle? near path? that was already branched on?
            if let Some(expr) = lazy_required_bits_expr(
                self.graph,
                self.edges,
                self.first_node,
                self.required_bits,
                &value_graph,
            ) {
                // self.index += 1;
                // trace!("adding cut {}: {}", self.index, expr.name);
                // self.renderer.render_diff(
                //     format!("{}-cut.{}", self.index, EXT),
                //     &self.prev,
                //     &value_graph,
                // );
                // self.prev = value_graph;
                Some(expr)
            } else {
                self.index += 1;
                if self.index % 100 == 0 {
                    trace!("solved relaxation {}", self.index);
                }
                if self.index % 1000 == 0 {
                    self.renderer.render(
                        format!("{}-branch.{}", self.index, EXT),
                        &value_graph,
                        self.first_node,
                    );
                }
                None
            }
        }

        // fn get_heuristic_solution(&mut self, problem: &Prob) -> Option<Solution> {
        //     let value_graph = value_graph(self.graph, problem, self.edges);

        //     let mut visited = value_graph.visit_map();
        //     let mut stack = Vec::new();
        //     let mut path: Vec::new();
        //     stack.push(self.first_node);
        //     while let Some(next) = stack.pop() {
        //         if !visited.is_visited(&next) {
        //             visited.visit(next);
        //             value_graph
        //                 .edges(next)
        //                 .filter(|e| *e.weight() > EPS)
        //                 .filter(|e| {
        //                     value_graph
        //                         .edges_directed(e.target(), Incoming)
        //                         .filter(|e| *e.weight() > EPS)
        //                         .all(|e| visited.is_visited(&e.source()))
        //                 })
        //                 .sorted_by(|l, r| l.weight().partial_cmp(r.weight()).unwrap());
        //         }
        //     }
        //     None
        // }

        fn get_branch(&mut self, problem: &Prob) -> Option<(VarRef, Branch)> {
            fn weight_score(value: f64) -> f64 {
                (value - 0.5).abs()
            }
            fn index_score(index: usize) -> f64 {
                index as f64 * 0.01
            }

            let value_graph = value_graph(self.graph, problem, self.edges);

            #[derive(Clone, Debug, Default)]
            struct State<'g> {
                edge: Option<EdgeReference<'g, f64>>,
                bits: i32,
                time: f64,
            }
            let mut states = HashMap::<NodeIndex, State>::new();
            DfsPostOrder::new(&value_graph, self.first_node)
                .iter(&value_graph)
                .for_each(|n| {
                    let last_bits = self.graph[n].bits;
                    let state = value_graph
                        .edges(n)
                        .filter_map(|e| {
                            states.get(&e.target()).map(|next| State {
                                edge: Some(e),
                                bits: last_bits + next.bits,
                                time: self.graph[e.id()].get_frames() + next.time,
                            })
                        })
                        .max_by(|l, r| {
                            l.bits
                                .cmp(&r.bits)
                                .then(l.time.partial_cmp(&r.time).unwrap().reverse())
                        })
                        .unwrap_or_default();
                    states.insert(n, state);
                });
            iter::successors(states.get(&self.first_node), |&s| {
                s.edge.and_then(|e| states.get(&e.target()))
            })
            .filter_map(|s| s.edge)
            .filter(|e| 1.0 - *e.weight() > EPS)
            .enumerate()
            .map(|(i, e)| {
                let score = weight_score(*e.weight()) + index_score(i);
                (e.id(), score)
            })
            .min_by(|l, r| l.1.partial_cmp(&r.1).unwrap())
            .map(|(e, _)| (self.edges.get(e.index()), Branch::Up))
        }

        fn new_best_solution(&mut self, problem: &Prob) {
            self.index += 1;
            info!("new best solution {}", self.index);
            self.renderer.render(
                format!("{}-SOLVE.{}", self.index, EXT),
                &value_graph(self.graph, problem, self.edges),
                self.first_node,
            );
        }
    }

    problem.optimize_mip(&mut closure).unwrap();

    closure.index += 1;
    closure.renderer.render(
        format!("{}-BEST.{}", closure.index, EXT),
        &value_graph_int(graph, &problem, edges),
        closure.first_node,
    );
    trace!("done!");
}

#[test]
fn order() {
    /*
             / 2 \ /5\
        0 - 1     4 - 6 = 7 - 8   9 = 10
             \ 3 /
    */
    let graph: StableGraph<(), ()> = StableGraph::from_edges(&[
        (0, 1),
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 4),
        (4, 5),
        (4, 6),
        (5, 6),
        (6, 7),
        (7, 6),
        (7, 8),
        (9, 10),
        (10, 9),
    ]);
    let first = NodeIndex::new(0);

    fn longest_simple<'g>(
        graph: &'g StableGraph<(), ()>,
        first: NodeIndex,
    ) -> Option<Vec<NodeIndex>> {
        let mut simple = DfsPostOrder::new(graph, first).iter(graph).collect_vec();
        if simple
            .iter()
            .tuple_windows()
            .all(|(&target, &source)| graph.find_edge(source, target).is_some())
        {
            simple.reverse();
            Some(simple)
        } else {
            None
        }
    }

    // TODO does this ever find a path with enough bits when simple doesn't?
    fn longext_complex<'g>(graph: &'g StableGraph<(), ()>, first: NodeIndex) -> Vec<NodeIndex> {
        #[derive(Debug, Clone, Copy, Default)]
        struct State {
            // TODO might also want the edge index? for now just re-find it
            next: Option<NodeIndex>,
            bits: i32,
        }
        let mut paths = HashMap::<NodeIndex, State>::new();
        for i in DfsPostOrder::new(graph, first).iter(graph) {
            let state = graph
                .edges(i)
                .filter_map(|e| {
                    paths.get(&e.target()).map(|s| State {
                        next: Some(e.target()),
                        bits: s.bits + 1, // assuming all nodes have 1 bit for testing
                    })
                })
                .max_by_key(|s| s.bits)
                .unwrap_or_default();
            paths.insert(i, state);
        }
        std::iter::successors(Some(first), |prev| {
            paths.get(prev).and_then(|state| state.next)
        })
        .collect_vec()
    }

    longest_simple(&graph, first);
}

fn value_graph<'g>(
    graph: &'g StableGraph<Node, Edge>,
    problem: &Prob,
    edges: VarRefs,
) -> StableGraph<&'g Node, f64> {
    let filtered = graph.filter_map(
        |_, n| Some(n),
        |i, _| {
            let value = problem.get_value(edges.get(i.index()));
            if value > EPS {
                Some(value)
            } else {
                None
            }
        },
    );

    graph.edge_references().for_each(|g| {
        if let Some((fs, ft)) = filtered.edge_endpoints(g.id()) {
            let gs = g.source();
            let gt = g.target();
            if gs != fs || gt != ft {
                panic!(
                    "edge id {} changed from {}->{} to {}->{}",
                    g.id().index(),
                    gs.index(),
                    gt.index(),
                    fs.index(),
                    ft.index()
                );
            }
        }
    });

    filtered
}

fn value_graph_int<'g>(
    graph: &'g StableGraph<Node, Edge>,
    problem: &Prob,
    edges: VarRefs,
) -> StableGraph<&'g Node, f64> {
    graph.filter_map(
        |_, n| Some(n),
        |i, _| {
            let value = problem.get_int_value(edges.get(i.index()));
            if value > EPS {
                Some(value)
            } else {
                None
            }
        },
    )
}

fn edge_vars(graph: &StableGraph<Node, Edge>) -> Vec<Var> {
    graph
        .edge_references()
        .map(|e| {
            let edge = e.weight();
            let source = &graph[e.source()];
            let target = &graph[e.target()];
            Var {
                name: format!("{}/to/{}", source.name, target.name),
                kind: Kind::Int,
                bounds: Bounds::Double(0.0, 1.0),
                objective: edge.get_frames() + target.time,
            }
        })
        .collect()
}

fn flow_exprs(
    graph: &StableGraph<Node, Edge>,
    edges: VarRefs,
    first_node: NodeIndex,
    last_node: NodeIndex,
) -> Vec<Expr> {
    graph
        .node_references()
        .map(|(n, node)| Expr {
            name: format!("{}/flow", node.name),
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
    graph: &StableGraph<Node, Edge>,
    edges: VarRefs,
    first_node: NodeIndex,
    last_node: NodeIndex,
) -> Vec<Expr> {
    graph
        .node_references()
        .filter(|&(n, _)| n != first_node && n != last_node)
        .map(|(n, node)| Expr {
            name: format!("{}/capacity", node.name),
            bounds: Bounds::Upper(1.0),
            terms: graph
                .edges_directed(n, Outgoing)
                .map(|e| edges.get(e.id().index()) * 1.0)
                .collect(),
        })
        .collect()
}

fn no_2_cycles(graph: &StableGraph<Node, Edge>, edges: VarRefs) -> Vec<Expr> {
    graph
        .edge_references()
        .filter(|e| e.source().index() < e.target().index())
        .filter_map(|e| graph.find_edge(e.target(), e.source()).map(|e2| (e, e2)))
        .map(|(a, b)| Expr {
            name: format!(
                "{}/{}/cycle",
                graph[a.source()].name,
                graph[a.target()].name
            ),
            bounds: Bounds::Upper(1.0),
            terms: vec![edges.get(a.id().index()) * 1.0, edges.get(b.index()) * 1.0],
        })
        .collect()
}

fn no_3_cycles(graph: &StableGraph<Node, Edge>, edges: VarRefs) -> Vec<Expr> {
    graph
        .node_references()
        .flat_map(|(i, n)| {
            let sources = graph
                .edges_directed(i, Incoming)
                .filter(move |e| i.index() < e.source().index());
            // TODO StableGraph.edges_directed doesn't implement Clone
            let targets = graph
                .edges_directed(i, Outgoing)
                .filter(move |e| i.index() < e.target().index());
            sources
                .cartesian_product(targets)
                .filter_map(|(source_edge, target_edge)| {
                    graph
                        .find_edge(target_edge.target(), source_edge.source())
                        .map(|opposite_edge| (source_edge, target_edge, opposite_edge))
                })
                .map(move |(s, t, o)| Expr {
                    name: format!(
                        "{}/{}/{}/cycle",
                        graph[s.source()].name,
                        n.name,
                        graph[t.target()].name
                    ),
                    bounds: Bounds::Upper(2.0),
                    terms: vec![
                        edges.get(s.id().index()) * 1.0,
                        edges.get(t.id().index()) * 1.0,
                        edges.get(o.index()) * 1.0,
                    ],
                })
        })
        .collect()
}

/// not bothering with other required_bits nodes yet since they shouldn't be violated based on timing data
fn required_bits_expr(graph: &StableGraph<Node, Edge>, edges: VarRefs, required_bits: i32) -> Expr {
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

fn total_keys_expr(graph: &StableGraph<Node, Edge>, edges: VarRefs) -> Expr {
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

// fn approx_water_lock_exprs(graph: &StableGraph<Node, Edge>, edges: VarRefs) -> Vec<Expr> {
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
    graph: &StableGraph<Node, Edge>,
    edges: VarRefs,
    first_node: NodeIndex,
    required_bits: i32,
    values: &StableGraph<&Node, f64>,
) -> Option<Expr> {
    let (connected_nodes, connected_bits) = get_connected_nodes(values, first_node);
    if connected_bits < required_bits {
        Some(Expr {
            name: format!("cut.{:?}", connected_nodes),
            bounds: Bounds::Lower(1.0),
            terms: connected_nodes
                .ones()
                .map(NodeIndex::new)
                .flat_map(|n| {
                    graph
                        .edges_directed(n, Outgoing)
                        .filter(|e| !connected_nodes.contains(e.target().index()))
                        .map(|e| edges.get(e.id().index()) * 1.0)
                })
                .collect(),
        })
    } else {
        None
    }
}

fn get_connected_nodes(
    values: &StableGraph<&Node, f64>,
    first_node: NodeIndex,
) -> (FixedBitSet, i32) {
    let mut connected_bits = 0;
    let mut dfs = Dfs::new(&values, first_node);
    while let Some(n) = dfs.next(values) {
        connected_bits += values[n].bits;
    }
    (dfs.discovered, connected_bits)
}
