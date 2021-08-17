mod opt;
mod render;
mod rooms;
// mod value;

use itertools::Itertools;
use log::*;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{
    DfsPostOrder, EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoNodeReferences, Walker,
};
use petgraph::EdgeDirection::Incoming;
use rooms::Cost;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let graph = rooms::load("fez-route/rooms.json");
    opt::optimize(&graph, 32 * 8);
}

#[test]
fn load() {
    SimpleLogger::new().init().unwrap();

    let graph = rooms::load("../fez-route/rooms.json");

    let first = graph.externals(Incoming).next().unwrap();

    let no_secret_doors = EdgeFiltered::from_fn(&graph, |e| e.weight().cost != Some(Cost::Secret));
    DfsPostOrder::new(&no_secret_doors, first)
        .iter(&no_secret_doors)
        .for_each(|n| info!("{}", graph[n].name));
}
