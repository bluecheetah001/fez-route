mod common;
mod opt;
mod render;
mod rooms;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let graph = rooms::load("fez-route/rooms.json");
    opt::optimize(&graph, 16 * 8);
}

#[test]
fn load() {
    SimpleLogger::new().init().unwrap();

    rooms::load("../fez-route/rooms.json");
}
