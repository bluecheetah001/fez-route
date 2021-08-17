use itertools::Itertools;
use log::*;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/*

80 frames to open a door
60 frames to open a secret
370 frames to lesser warp (not including a long load)
+40 frames to enter a hole
+140 frames to long load, not added yet
+230 frames to far load
268 frames to use well (including long load)
150 frames to open any chest
70 frames to spawn an anti
48 frames to collect any cube (avg 6 per bit)
170 frames to activate fork
200 frames to explode bomb, ignoring

15 frames to rotate
6ish frames per tile in both xz and y (12 for out and back)

*/

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum Orientation {
    Front,
    Back,
    Left,
    Right,
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
    orientation: Option<Orientation>,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Cost {
    Lock,
    Water,
    Secret,
}

#[derive(Deserialize, Debug, Clone)]
struct Collectable<'a> {
    name: &'a str,
    position: Position,
    #[serde(default)]
    bit: i32,
    #[serde(default)]
    cube: i32,
    #[serde(default)]
    anti: i32,
    #[serde(default)]
    key: i32,
    time: Option<f64>,
    cost: Option<Cost>,
    #[serde(skip, default = "NodeIndex::end")]
    index: NodeIndex,
}

#[derive(Deserialize, Debug, Clone)]
struct Door<'a> {
    to: Option<&'a str>,
    name: &'a str,
    position: Position,
    time: Option<f64>,
    cost: Option<Cost>,
    #[serde(skip, default = "NodeIndex::end")]
    index: NodeIndex,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct Room<'a> {
    name: &'a str,
    collectables: Vec<Collectable<'a>>,
    doors: Vec<Door<'a>>,
}

#[derive(Debug, Clone)]
pub struct Node {
    /// {room}.{name}.{to} for doors
    /// .{name}.{to} for dest
    /// {room}.{name} for collectables
    pub name: String,
    pub bits: i32,
    pub keys: i32,
    pub cost: Option<Cost>,
    pub time: f64,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub time: Distance,
    pub cost: Option<Cost>,
}

#[derive(Debug, Copy, Clone)]
pub struct Distance {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

pub fn load(path: impl AsRef<Path>) -> StableGraph<Node, Edge> {
    let mut s = String::new();
    File::open(path).unwrap().read_to_string(&mut s).unwrap();
    let mut rooms: Vec<Room> = serde_json::from_str(&s).unwrap();
    verify_unique_names(&rooms);
    check_for_improvements(&rooms);
    as_graph(&mut rooms)
}

// fractal has 1 doors and 4 collectables
// grave_ghost has 1 doors and 2 collectables
// industrial_city has 1 doors and 3 collectables
// mine_bomb_pillar has 1 doors and 2 collectables
// sewer_pivot has 1 doors and 2 collectables
// sewer_qr has 1 doors and 2 collectables
// stargate_ruins has 1 doors and 2 collectables
// wall_village has 1 doors and 4 collectables
// windmill_cave has 1 doors and 2 collectables
// zu_house_empty has 1 doors and 2 collectables
// zu_throne_ruins has 1 doors and 2 collectables
fn check_for_improvements(rooms: &[Room]) {
    rooms.iter().for_each(|room| {
        if room.doors.len() == 1 && room.collectables.len() >= 1 {
            warn!(
                "{} has {} doors and {} collectables",
                room.name,
                room.doors.len(),
                room.collectables.len()
            );
        }
    })
}

fn verify_unique_names(rooms: &[Room]) {
    verify_unique_room_names(rooms);
    rooms.iter().for_each(verify_unique_inner_names);
}

fn verify_unique_room_names(rooms: &[Room]) {
    rooms.iter().tuple_combinations().for_each(|(a, b)| {
        if a.name == b.name {
            warn!("multiple definitions for room {}", a.name);
        }
    });
}

fn verify_unique_inner_names(room: &Room) {
    room.collectables
        .iter()
        .tuple_combinations()
        .for_each(|(a, b)| {
            if a.name == b.name {
                warn!(
                    "multiple definitions for collectable {}.{}",
                    room.name, a.name
                );
            }
        });
    room.doors.iter().tuple_combinations().for_each(|(a, b)| {
        if a.name == b.name {
            match (a.to.as_ref(), b.to.as_ref()) {
                (Some(a_to), Some(b_to)) => {
                    if a_to == b_to {
                        warn!(
                            "multiple definitions for door {}.{}.{}",
                            room.name, a.name, a_to
                        );
                    }
                }
                (None, None) => {
                    warn!("multiple definitions for dest {}.{}", room.name, a.name);
                }
                // dests are used preferentially to doors
                _ => {}
            }
        }
    });
    room.doors
        .iter()
        .filter(|door| door.to.is_none())
        .cartesian_product(room.collectables.iter())
        .for_each(|(dest, collectable)| {
            if dest.name == collectable.name {
                warn!("collectable {}.{} is also a dest", room.name, dest.name);
            }
        });
}

fn as_graph(rooms: &mut [Room]) -> StableGraph<Node, Edge> {
    let mut graph = StableGraph::new();
    rooms
        .iter_mut()
        .for_each(|room| add_room_nodes(&mut graph, room));
    rooms
        .iter()
        .for_each(|room| add_room_edges(&mut graph, rooms, room));
    graph
}

fn add_room_nodes(graph: &mut StableGraph<Node, Edge>, room: &mut Room) {
    for collectable in &mut room.collectables {
        let bits = collectable.bit + (collectable.cube + collectable.anti) * 8;
        collectable.index = graph.add_node(Node {
            name: format!("{}.{}", room.name, collectable.name),
            bits,
            keys: collectable.key,
            cost: collectable.cost,
            // TODO this is pretty accurate, go ahead and put this everywhere
            time: collectable.time.unwrap_or(bits as f64 * 6.0),
        });
    }
    for door in &mut room.doors {
        if let Some(to) = door.to {
            door.index = graph.add_node(Node {
                name: format!("{}.{}.{}", room.name, door.name, to),
                bits: 0,
                keys: 0,
                cost: door.cost,
                // TODO this is not accurate, maybe could infer some based on cost and/or name
                //      but really just need to check them all
                time: door.time.unwrap_or(40.0),
            });
        }
    }
}

fn add_room_edges(graph: &mut StableGraph<Node, Edge>, rooms: &[Room], room: &Room) {
    for collectable in &room.collectables {
        add_edges(
            graph,
            collectable.index,
            collectable.position,
            room,
            collectable.index,
        );
    }
    for door in &room.doors {
        if let Some(to) = door.to {
            if let Some(to) = rooms.iter().find(|&room| room.name == to) {
                if let Some(rev) = to
                    .doors
                    .iter()
                    .filter(|&rev| rev.name == door.name)
                    .filter(|&rev| rev.to.map_or(true, |rev_to| rev_to == room.name))
                    .min_by_key(|&rev| rev.to.is_some())
                {
                    add_edges(graph, door.index, rev.position, to, rev.index);
                } else {
                    warn!("no dest door for {}.{}.{}", room.name, door.name, to.name)
                }
            } else {
                warn!("no dest room for {}.{}.{}", room.name, door.name, to)
            }
        }
    }
}

fn add_edges(
    graph: &mut StableGraph<Node, Edge>,
    src_i: NodeIndex,
    src_pos: Position,
    room: &Room,
    exclude: NodeIndex,
) {
    room.collectables
        .iter()
        .map(|c| (c.index, c.position, c.cost))
        .chain(
            room.doors
                .iter()
                .filter(|&d| d.to.is_some())
                .map(|d| (d.index, d.position, d.cost)),
        )
        .filter(|&(i, _, _)| i != exclude)
        .for_each(|(dest_i, dest_pos, cost)| {
            graph.add_edge(
                src_i,
                dest_i,
                Edge {
                    time: Distance {
                        dx: dest_pos.x - src_pos.x,
                        dy: dest_pos.y - src_pos.y,
                        dz: dest_pos.z - src_pos.z,
                    },
                    cost,
                },
            );
        })
}
