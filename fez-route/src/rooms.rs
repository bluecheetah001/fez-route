use itertools::Itertools;
use log::*;
use petgraph::graph::{Graph, NodeIndex};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Cost {
    Lock,
    Water,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Time {
    Chest,
    Puzzle,
    Far,
}

#[derive(Deserialize, Debug)]
struct Collectable<'a> {
    name: &'a str,
    position: Position,
    #[serde(default)]
    bit: f64,
    #[serde(default)]
    cube: f64,
    #[serde(default)]
    anti: f64,
    #[serde(default)]
    key: f64,
    time: Option<Time>,
    cost: Option<Cost>,
    #[serde(skip, default = "NodeIndex::end")]
    index: NodeIndex,
}

#[derive(Deserialize, Debug)]
struct Door<'a> {
    to: Option<&'a str>,
    name: &'a str,
    position: Position,
    time: Option<Time>,
    cost: Option<Cost>,
    #[serde(skip, default = "NodeIndex::end")]
    index: NodeIndex,
}

#[derive(Deserialize, Debug, Default)]
struct Room<'a> {
    name: &'a str,
    collectables: Vec<Collectable<'a>>,
    doors: Vec<Door<'a>>,
}

#[derive(Debug)]
pub struct Node<T> {
    // these names are highly repetative
    // but interning to avoid allocations and get faster eq is annoying
    pub room_name: String,
    pub name: String,
    pub to_name: Option<String>,
    pub bit: f64,
    pub cube: f64,
    pub anti: f64,
    pub key: f64,
    pub cost: Option<Cost>,
    pub time: T,
}

#[derive(Debug)]
pub struct Edge<T> {
    time: T,
}

#[derive(Debug, Copy, Clone)]
pub struct Distance {
    dx: f64,
    dy: f64,
    dz: f64,
}

pub fn load(path: impl AsRef<Path>) -> Graph<Node<Option<Time>>, Edge<Distance>> {
    let mut s = String::new();
    File::open(path).unwrap().read_to_string(&mut s).unwrap();
    let mut rooms: Vec<Room> = serde_json::from_str(&s).unwrap();
    verify_unique_names(&rooms);
    as_graph(&mut rooms)
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

fn as_graph(rooms: &mut [Room]) -> Graph<Node<Option<Time>>, Edge<Distance>> {
    let mut graph = Graph::new();
    rooms
        .iter_mut()
        .for_each(|room| add_room_nodes(&mut graph, room));
    rooms
        .iter()
        .for_each(|room| add_room_edges(&mut graph, rooms, room));
    graph
}

fn add_room_nodes(graph: &mut Graph<Node<Option<Time>>, Edge<Distance>>, room: &mut Room) {
    for collectable in &mut room.collectables {
        collectable.index = graph.add_node(Node {
            room_name: room.name.to_owned(),
            name: collectable.name.to_owned(),
            to_name: None,
            bit: collectable.bit,
            cube: collectable.cube,
            anti: collectable.anti,
            key: collectable.key,
            cost: collectable.cost,
            time: collectable.time,
        });
    }
    for door in &mut room.doors {
        if let Some(to) = door.to {
            door.index = graph.add_node(Node {
                room_name: room.name.to_owned(),
                name: door.name.to_owned(),
                to_name: Some(to.to_owned()),
                bit: 0.0,
                cube: 0.0,
                anti: 0.0,
                key: 0.0,
                cost: door.cost,
                time: door.time,
            });
        }
    }
}

fn add_room_edges(
    graph: &mut Graph<Node<Option<Time>>, Edge<Distance>>,
    rooms: &[Room],
    room: &Room,
) {
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
    graph: &mut Graph<Node<Option<Time>>, Edge<Distance>>,
    src_i: NodeIndex,
    src_pos: Position,
    room: &Room,
    except: NodeIndex,
) {
    room.collectables
        .iter()
        .map(|c| (c.index, c.position))
        .chain(room.doors.iter().map(|d| (d.index, d.position)))
        .filter(|&(dest_i, _)| dest_i != NodeIndex::end() && dest_i != except)
        .for_each(|(dest_i, dest_pos)| {
            graph.add_edge(
                src_i,
                dest_i,
                Edge {
                    time: Distance {
                        dx: dest_pos.x - src_pos.x,
                        dy: dest_pos.y - src_pos.y,
                        dz: dest_pos.z - src_pos.z,
                    },
                },
            );
        })
}
