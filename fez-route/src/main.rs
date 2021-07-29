mod fez;
mod ll;
mod rooms;

use ll::{optimize, Edge, Node};
use log::*;
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use petgraph::visit::{
    Dfs, EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeRef,
};
use petgraph::Direction::{Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64;
use simple_logger::SimpleLogger;

const NUM_NODES: usize = 60;
const X_SIZE: i32 = 10;
const Y_SIZE: i32 = 30;
const MIN_BITS: i32 = 32;
const MAX_DIST_X: i32 = 1;
const MAX_DIST_Y: i32 = 15;
const BIT_WEIGHTS: [i32; 9] = [10, 10, 0, 0, 0, 0, 0, 0, 5];
fn generate_example() -> Graph<Node, Edge> {
    let mut rng = Pcg64::seed_from_u64(8097123498761234);
    let x_range = Uniform::new(0, X_SIZE);
    let y_range = Uniform::new(0, Y_SIZE);
    let bits = WeightedIndex::new(&BIT_WEIGHTS).unwrap().map(|i| i as i32);
    let mut positions: Vec<(i32, i32)> = Vec::new();
    let mut graph = Graph::new();
    for i in 0..NUM_NODES {
        let x = rng.sample(&x_range);
        let y = rng.sample(&y_range);
        let b = rng.sample(&bits);
        positions.push((x, y));
        graph.add_node(Node {
            name: format!("{}-{}", x, y),
            bits: b,
            ..Node::default()
        });
        trace!("node i:{:02} x:{:02} y:{:02} b:{}", i, x, y, b);
    }
    for source_i in 0..NUM_NODES - 1 {
        let (source_x, source_y) = positions[source_i];
        for target_i in 1..NUM_NODES {
            if source_i != target_i {
                let (target_x, target_y) = positions[target_i];
                let dx = (source_x - target_x).abs();
                let dy = (source_y - target_y).abs();
                let dist_2 = dx * dx + dy * dy;
                if dx <= MAX_DIST_X && dy <= MAX_DIST_Y {
                    let frames = if dist_2 <= 1 {
                        1
                    } else {
                        rng.gen_range(dist_2 / 2..dist_2)
                    };
                    graph.add_edge(
                        NodeIndex::new(source_i),
                        NodeIndex::new(target_i),
                        Edge { frames },
                    );
                    trace!("edge {:02}->{:02} f:{:02}", source_i, target_i, frames);
                }
            }
        }
    }
    graph
}

fn main() {
    SimpleLogger::new().init().unwrap();

    fez::render("rooms.dot");

    if false {
        let graph = generate_example();
        optimize(&graph, MIN_BITS);
    }
}
