use std::iter::Enumerate;

use crate::rooms::{Edge, Node};
use fixedbitset::FixedBitSet;
use petgraph::stable_graph::{EdgeIndex, NodeIndex, NodeIndices, StableGraph};
use petgraph::visit::{
    Bfs, Data, Dfs, DfsPostOrder, EdgeFiltered, EdgeIndexable, EdgeRef, GraphBase, GraphProp,
    GraphRef, IntoEdgeReferences, IntoEdges, IntoEdgesDirected, IntoNeighbors,
    IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable,
    NodeCount, NodeIndexable, NodeRef, Reversed, Topo, VisitMap, Visitable, Walker,
};
use petgraph::Direction::{self, Incoming, Outgoing};
use petgraph::{Directed, EdgeDirection, EdgeType};

pub const EPS: f64 = 1e-6;

#[derive(Debug, Clone)]
pub struct ValueGraph<'g> {
    pub original: &'g StableGraph<Node, Edge>,
    values: Vec<f64>,
}
impl<'g> ValueGraph<'g> {
    pub fn new(original: &'g StableGraph<Node, Edge>, get: impl Fn(EdgeIndex) -> f64) -> Self {
        Self {
            original,
            values: original.edge_indices().map(get).collect(),
        }
    }
}
impl GraphBase for ValueGraph<'_> {
    type EdgeId = EdgeIndex;
    type NodeId = NodeIndex;
}
impl GraphProp for ValueGraph<'_> {
    type EdgeType = Directed;
}
impl<'g> Data for &'g ValueGraph<'g> {
    type NodeWeight = &'g Node;
    type EdgeWeight = f64;
}
impl<'g> Visitable for ValueGraph<'g> {
    type Map = FixedBitSet;

    fn visit_map(self: &Self) -> Self::Map {
        self.original.visit_map()
    }
    fn reset_map(self: &Self, map: &mut Self::Map) {
        self.original.reset_map(map)
    }
}

impl NodeCount for ValueGraph<'_> {
    fn node_count(self: &Self) -> usize {
        self.original.node_count()
    }
}
impl NodeIndexable for ValueGraph<'_> {
    fn node_bound(self: &Self) -> usize {
        self.node_count()
    }

    fn to_index(self: &Self, i: Self::NodeId) -> usize {
        i.index()
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        NodeIndex::new(i)
    }
}
impl NodeCompactIndexable for ValueGraph<'_> {}

impl<'g> IntoNodeIdentifiers for &'g ValueGraph<'g> {
    type NodeIdentifiers = NodeIndices;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.original.node_identifiers()
    }
}
impl<'g> IntoNodeReferences for &'g ValueGraph<'g> {
    type NodeRef = NodeReference<'g>;
    type NodeReferences = NodeReferences<'g>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences {
            iter: self.original.node_references(),
        }
    }
}
impl<'g> IntoNeighbors for &'g ValueGraph<'g> {
    type Neighbors = Neighbors<'g>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.neighbors_directed(n, Outgoing)
    }
}
impl<'g> IntoNeighborsDirected for &'g ValueGraph<'g> {
    type NeighborsDirected = Neighbors<'g>;

    fn neighbors_directed(self, n: Self::NodeId, d: EdgeDirection) -> Self::NeighborsDirected {
        todo!()
    }
}

impl EdgeIndexable for ValueGraph<'_> {
    fn edge_bound(self: &Self) -> usize {
        self.original.edge_count()
    }

    fn to_index(self: &Self, i: Self::EdgeId) -> usize {
        i.index()
    }

    fn from_index(self: &Self, i: usize) -> Self::EdgeId {
        EdgeIndex::new(i)
    }
}
impl<'g> IntoEdgeReferences for &'g ValueGraph<'g> {
    type EdgeRef = EdgeReference;
    type EdgeReferences = EdgeReferences<'g>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            original: self.original,
            iter: self.values.iter().enumerate(),
        }
    }
}
impl<'g> IntoEdges for &'g ValueGraph<'g> {
    type Edges = Edges<'g>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        todo!()
    }
}
impl<'g> IntoEdgesDirected for &'g ValueGraph<'g> {
    type EdgesDirected = Edges<'g>;

    fn edges_directed(self, a: Self::NodeId, dir: EdgeDirection) -> Self::EdgesDirected {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NodeReference<'g> {
    index: NodeIndex,
    weight: &'g Node,
}
impl<'g> NodeRef for NodeReference<'g> {
    type NodeId = NodeIndex;
    type Weight = &'g Node;
    fn id(&self) -> Self::NodeId {
        self.index
    }
    fn weight(&self) -> &Self::Weight {
        &self.weight
    }
}
#[derive(Clone, Debug)]
pub struct NodeReferences<'g> {
    iter: petgraph::graph::NodeReferences<'g, Node>,
}
impl<'g> Iterator for NodeReferences<'g> {
    type Item = NodeReference<'g>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(index, weight)| NodeReference { index, weight })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Debug)]
pub struct Neighbors<'g> {
    iter: EdgeReferences<'g>,
    dir: Direction,
}
impl Iterator for Neighbors<'_> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| match self.dir {
            Incoming => e.source(),
            Outgoing => e.target(),
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeReference {
    index: EdgeIndex,
    source: NodeIndex,
    target: NodeIndex,
    weight: f64,
}
impl EdgeRef for EdgeReference {
    type NodeId = NodeIndex;
    type EdgeId = EdgeIndex;
    type Weight = f64;

    fn id(&self) -> Self::EdgeId {
        self.index
    }
    fn source(&self) -> Self::NodeId {
        self.source
    }
    fn target(&self) -> Self::NodeId {
        self.target
    }
    fn weight(&self) -> &Self::Weight {
        &self.weight
    }
}

#[derive(Clone, Debug)]
pub struct EdgeReferences<'g> {
    original: &'g StableGraph<Node, Edge>,
    iter: Enumerate<std::slice::Iter<'g, f64>>,
}
impl<'g> Iterator for EdgeReferences<'g> {
    type Item = EdgeReference;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, &weight)) = self.iter.next() {
            if weight > EPS {
                let index = EdgeIndex::new(index);
                let (source, target) = self.original.edge_endpoints(index).unwrap();
                return Some(EdgeReference {
                    index,
                    source,
                    target,
                    weight,
                });
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

#[derive(Clone, Debug)]
pub struct EdgeReferences<'g> {
    original: &'g StableGraph<Node, Edge>,
    iter: Enumerate<std::slice::Iter<'g, f64>>,
}
impl<'g> Iterator for EdgeReferences<'g> {
    type Item = EdgeReference;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, &weight)) = self.iter.next() {
            if weight > EPS {
                let index = EdgeIndex::new(index);
                let (source, target) = self.original.edge_endpoints(index).unwrap();
                return Some(EdgeReference {
                    index,
                    source,
                    target,
                    weight,
                });
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}
