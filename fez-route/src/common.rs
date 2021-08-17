use crate::render::{Renderer, EXT};
use crate::rooms::{Cost, Edge, Node};
use itertools::Itertools;
use log::*;
use petgraph::stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph};
use petgraph::visit::{
    Dfs, DfsPostOrder, EdgeFiltered, EdgeRef, GraphBase, GraphRef, IntoEdgeReferences, IntoEdges,
    IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
    IntoNodeReferences, NodeRef, VisitMap, Visitable, Walker,
};
use petgraph::Direction::{Incoming, Outgoing};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct State<'g> {
    edge: Option<EdgeReference<'g, f64>>,
    bits: i32,
    weight: f64,
}

#[derive(Clone, Debug)]
pub struct HeuristicPath<'g> {
    states: HashMap<NodeIndex, State<'g>>,
    first: NodeIndex,
}
impl<'p, 'g> IntoIterator for &'p HeuristicPath<'g> {
    type Item = EdgeReference<'g, f64>;
    type IntoIter = HeuristicPathIter<'p, 'g>;

    fn into_iter(self) -> Self::IntoIter {
        HeuristicPathIter {
            states: &self.states,
            node: self.first,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HeuristicPathIter<'p, 'g> {
    states: &'p HashMap<NodeIndex, State<'g>>,
    node: NodeIndex,
}
impl<'p, 'g> Iterator for HeuristicPathIter<'p, 'g> {
    type Item = EdgeReference<'g, f64>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(state) = self.states.get(&self.node) {
            if let Some(edge) = state.edge {
                self.node = edge.target();
                return Some(edge);
            }
        }
        None
    }
}

pub fn heuristic_path<'g>(
    values: &'g StableGraph<&'g Node, f64>,
    first: NodeIndex,
    last: NodeIndex,
) -> HeuristicPath<'g> {
    let mut states = HashMap::<NodeIndex, State>::new();
    states.insert(last, State::default());
    DfsPostOrder::new(&values, first)
        .iter(&values)
        .for_each(|n| {
            let last_bits = values[n].bits;
            if let Some(state) = values
                .edges(n)
                .filter_map(|e| {
                    states.get(&e.target()).map(|next| State {
                        edge: Some(e),
                        bits: last_bits + next.bits,
                        weight: values[e.id()] + next.weight,
                    })
                })
                .max_by(|l, r| {
                    l.bits
                        .cmp(&r.bits)
                        .then(l.weight.partial_cmp(&r.weight).unwrap())
                })
            {
                states.insert(n, state);
            }
        });
    HeuristicPath { states, first }
}
