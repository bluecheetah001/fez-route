mod opt;
mod render;
mod rooms;
// mod value;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let graph = rooms::load("fez-route/rooms.json");
    opt::optimize(&graph, 8 * 8);
}

#[test]
fn load() {
    SimpleLogger::new().init().unwrap();

    rooms::load("../fez-route/rooms.json");
}
