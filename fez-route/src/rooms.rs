use itertools::Itertools;
use log::*;
use petgraph::graph::{Graph, NodeIndex};
use serde::de::{Unexpected, Visitor};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/*

160 frames to open a door
120 frames to open a secret
+690 frames to warp (not including long load)
+80 frames to enter a hole
+290 frames to long load, not added yet
+460 frames to far load
240 frames to use well (not including long load)
300 frames to open any chest
135 frames to spawn an anti
96 frames to collect any cube (avg 12 per bit)
320 frames to activate fork (including rotates)

30  frames to rotate
12ish frames per tile in both xz and y (24 for out and back)

grave dest:

*/

#[derive(Deserialize, Debug, Default, Clone)]
struct Room<'a> {
    name: &'a str,
    nodes: Vec<RoomNode<'a>>,
}

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

#[derive(Debug, Copy, Clone)]
enum RoomTime {
    Unknown,
    Src,
    Start,
    End,
    Time(f64),
}
impl Default for RoomTime {
    fn default() -> Self {
        Self::Unknown
    }
}
impl<'de> Deserialize<'de> for RoomTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RoomTimeVisitor;
        impl<'de> Visitor<'de> for RoomTimeVisitor {
            type Value = RoomTime;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a number or oneof \"src\", \"start\", \"end\"")
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                self.visit_f64(v as f64)
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                self.visit_f64(v as f64)
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(RoomTime::Time(v))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "src" => Ok(RoomTime::Src),
                    "start" => Ok(RoomTime::Start),
                    "end" => Ok(RoomTime::End),
                    _ => Err(E::invalid_value(
                        Unexpected::Str(v),
                        &"oneof \"src\", \"start\", \"end\"",
                    )),
                }
            }
        }
        deserializer.deserialize_any(RoomTimeVisitor)
    }
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Cost {
    Free,
    Lock,
    Water,
    Secret,
    // at the moment there is only a single oneof constraint
    // so this doesn't need an arg
    Oneof,
}
impl Default for Cost {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Deserialize, Debug, Clone)]
struct RoomNode<'a> {
    name: &'a str,
    to: Option<&'a str>,
    // at: &'a str,
    position: Position,
    #[serde(default)]
    bit: i32,
    #[serde(default)]
    cube: i32,
    #[serde(default)]
    anti: i32,
    #[serde(default)]
    key: i32,
    #[serde(default)]
    time: RoomTime,
    #[serde(default)]
    cost: Cost,
    #[serde(skip, default = "NodeIndex::end")]
    index: NodeIndex,
}
impl RoomNode<'_> {
    fn is_actual(&self) -> bool {
        !matches!(self.time, RoomTime::Src)
    }
    fn is_source(&self) -> bool {
        !matches!(self.time, RoomTime::Src | RoomTime::End)
    }
    fn is_target(&self) -> bool {
        !matches!(self.time, RoomTime::Src | RoomTime::Start)
    }
    fn get_time(&self) -> f64 {
        match self.time {
            // TODO we need something, so for now assume the time to go through a hole
            // all collectables should have an actual time
            RoomTime::Unknown => 80.0,
            RoomTime::Time(time) => time,
            _ => 0.0,
        }
    }
    fn get_bits(&self) -> i32 {
        self.bit + self.cube * 8 + self.anti * 8
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    /// {room}.{name}
    pub name: String,
    pub bits: i32,
    pub keys: i32,
    pub cost: Cost,
    pub time: f64,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub time: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Distance {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

pub fn load(path: impl AsRef<Path>) -> Graph<Node, Edge> {
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
    room.nodes.iter().tuple_combinations().for_each(|(a, b)| {
        if a.name == b.name {
            warn!("multiple definitions for node {}.{}", room.name, a.name);
        }
    });
}

fn as_graph(rooms: &mut [Room]) -> Graph<Node, Edge> {
    let mut graph = Graph::new();
    rooms
        .iter_mut()
        .for_each(|room| add_room_nodes(&mut graph, room));
    let global = global_timing(rooms);
    rooms
        .iter()
        .for_each(|room| add_room_edges(&mut graph, rooms, room, &room_timing(room, &global)));
    graph
}

fn add_room_nodes(graph: &mut Graph<Node, Edge>, room: &mut Room) {
    let room_name = room.name;
    room.nodes
        .iter_mut()
        .filter(|node| node.is_actual())
        .for_each(|node| {
            node.index = graph.add_node(Node {
                name: format!("{}.{}", room_name, node.name),
                bits: node.get_bits(),
                keys: node.key,
                cost: node.cost,
                time: node.get_time(),
            })
        });
}

fn add_room_edges(graph: &mut Graph<Node, Edge>, rooms: &[Room], room: &Room, timing: &Timing) {
    room.nodes
        .iter()
        .filter(|node| node.is_source())
        .for_each(|source| {
            let to_name = source.to.unwrap_or(source.name);
            let to = rooms.iter().find(|r| r.name == to_name);
            if source.to.is_some() && to.is_none() {
                panic!(
                    "failed to find room {} for door {}.{}",
                    to_name, room.name, source.name
                );
            }
            if let Some(to) = to {
                let at_name = if source.to.is_some() {
                    source.name
                } else {
                    room.name
                };
                let at = to
                    .nodes
                    .iter()
                    .find(|n| n.name == at_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "failed to find node {}.{} for door {}.{}",
                            to.name, at_name, room.name, source.name
                        );
                    });
                add_edges(
                    graph,
                    source.index,
                    at.name,
                    at.position,
                    to,
                    at.index,
                    timing,
                );
            } else {
                add_edges(
                    graph,
                    source.index,
                    source.name,
                    source.position,
                    room,
                    source.index,
                    timing,
                );
            }
        });
}

fn add_edges<'a>(
    graph: &mut Graph<Node, Edge>,
    src_i: NodeIndex,
    src_name: &'a str,
    src_pos: Position,
    room: &Room,
    exclude: NodeIndex,
    timing: &Timing,
) {
    room.nodes
        .iter()
        .filter(|node| node.is_target())
        .filter(|node| node.index != exclude)
        .for_each(|target| {
            graph.add_edge(
                src_i,
                target.index,
                Edge {
                    time: timing.get(src_name, src_pos, target.name, target.position),
                },
            );
        });
}

struct GlobalTiming {}

struct Timing {}

fn global_timing(rooms: &[Room]) -> GlobalTiming {
    GlobalTiming {}
}

fn room_timing(room: &Room, global: &GlobalTiming) -> Timing {
    Timing {}
}

impl Timing {
    fn get(
        &self,
        src_name: &str,
        src_pos: Position,
        target_name: &str,
        target_pos: Position,
    ) -> f64 {
        let dx = (src_pos.x - target_pos.x).abs();
        let dy = (src_pos.y - target_pos.y).abs();
        let dz = (src_pos.z - target_pos.z).abs();
        (dx.min(dz) + dy) * 12.0
    }
}
