mod common;
mod opt;
mod render;
mod rooms;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut graph = rooms::load("fez-route/rooms.json");

    graph.retain_edges(|g, e| {
        let (source, target) = g.edge_endpoints(e).unwrap();
        let source = g[source].name.as_str();
        let target = g[target].name.as_str();

        // the only edge were we definately don't have a key
        if source == "gomez_house.start.villageville_3d" && target == "villageville_3d.boileroom" {
            return false;
        }

        // lighthouse is a diode
        if [
            "lighthouse.anti",
            "lighthouse.bit_1",
            "lighthouse.zu_fork",
            "lighthouse_house_a.door.lighthouse",
            "nature_hub.door.lighthouse",
        ]
        .contains(&source)
            && [
                "lighthouse.bit_2",
                "lighthouse.door.lighthouse_spin",
                "lighthouse.door.water_tower",
            ]
            .contains(&target)
        {
            return false;
        }

        return true;
    });

    opt::optimize(&graph.into(), 30 * 8);
}

#[test]
fn load() {
    SimpleLogger::new().init().unwrap();

    rooms::load("../fez-route/rooms.json");
}
