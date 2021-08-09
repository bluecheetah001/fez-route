mod opt;
mod render;
mod rooms;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut graph = rooms::load("fez-route/rooms.json");
    graph.retain_nodes(|g, i| {
        let n = &g[i];
        match n.name.as_str() {
            "villageville_3d.door.gomez_house" => false,
            "stargate.door.zu_city" => false,
            _ => true,
        }
    });
    opt::optimize(&graph, 8 * 8);
}

#[test]
fn load() {
    SimpleLogger::new().init().unwrap();

    rooms::load("../fez-route/rooms.json");
}
