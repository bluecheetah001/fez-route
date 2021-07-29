use itertools::Itertools;
use log::*;
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use petgraph::visit::{
    Dfs, EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeRef, VisitMap,
};
use petgraph::Direction::{Incoming, Outgoing};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug)]
pub struct Room {
    name: String,
    alias: String,
    bits: i32,
    cubes: i32,
    anti: i32,
    keys: i32,
    pos: Option<(f64, f64)>,
}
impl Default for Room {
    fn default() -> Self {
        Self {
            name: String::new(),
            alias: String::new(),
            cubes: 0,
            anti: 0,
            bits: 0,
            keys: 0,
            pos: None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Door {
    Door,
    Lock,
    Secret,
    SecretIndirect,
    SecretBi,
    Warp,
    Water,
    Owl,
}

pub fn rooms() -> Graph<Room, Door> {
    use self::Door::*;
    let mut graph = Graph::new();
    let abandoned_a = graph.add_node(Room {
        name: "abandoned_a".to_owned(),
        bits: 1, // not sure which abandoned
        ..Room::default()
    });
    let abandoned_b = graph.add_node(Room {
        name: "abandoned_b".to_owned(),
        bits: 1, // not sure which abandoned
        ..Room::default()
    });
    let abandoned_c = graph.add_node(Room {
        name: "abandoned_c".to_owned(),
        ..Room::default()
    });
    let ancient_walls = graph.add_node(Room {
        name: "ancient_walls".to_owned(),
        bits: 3,
        ..Room::default()
    });
    let arch = graph.add_node(Room {
        name: "arch".to_owned(),
        bits: 3,
        ..Room::default()
    });
    let bell_tower = graph.add_node(Room {
        name: "bell_tower".to_owned(),
        bits: 2,
        anti: 1,
        ..Room::default()
    });
    let big_owl = graph.add_node(Room {
        name: "big_owl".to_owned(),
        ..Room::default()
    });
    let big_tower = graph.add_node(Room {
        name: "big_tower".to_owned(),
        bits: 8,
        ..Room::default()
    });
    let boileroom = graph.add_node(Room {
        name: "boileroom".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let cabin_interior = graph.add_node(Room {
        name: "cabin_interior".to_owned(),
        alias: "cabin_interior_b".to_owned(),
        ..Room::default()
    });
    let clock = graph.add_node(Room {
        name: "clock".to_owned(),
        anti: 4,
        cubes: 1,
        ..Room::default()
    });
    let cmy = graph.add_node(Room {
        name: "cmy".to_owned(),
        ..Room::default()
    });
    let cmy_b = graph.add_node(Room {
        name: "cmy_b".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let cmy_fork = graph.add_node(Room {
        name: "cmy_fork".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let code_machine = graph.add_node(Room {
        name: "code_machine".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let crypt = graph.add_node(Room {
        name: "crypt".to_owned(),
        ..Room::default()
    });
    let extractor_a = graph.add_node(Room {
        name: "extractor_a".to_owned(),
        ..Room::default()
    });
    let five_towers = graph.add_node(Room {
        name: "five_towers".to_owned(),
        bits: 3,
        cubes: 1,
        ..Room::default()
    });
    let five_towers_cave = graph.add_node(Room {
        name: "five_towers_cave".to_owned(),
        ..Room::default()
    });
    let fox = graph.add_node(Room {
        name: "fox".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let fractal = graph.add_node(Room {
        name: "fractal".to_owned(),
        bits: 4,
        ..Room::default()
    });
    let geezer_house = graph.add_node(Room {
        name: "geezer_house".to_owned(),
        ..Room::default()
    });
    let globe = graph.add_node(Room {
        name: "globe".to_owned(),
        ..Room::default()
    });
    let globe_int = graph.add_node(Room {
        name: "globe_int".to_owned(),
        ..Room::default()
    });
    let gomez_house = graph.add_node(Room {
        name: "gomez_house".to_owned(),
        ..Room::default()
    });
    let grave_cabin = graph.add_node(Room {
        name: "grave_cabin".to_owned(),
        bits: 2,
        ..Room::default()
    });
    let grave_ghost = graph.add_node(Room {
        name: "grave_ghost".to_owned(),
        bits: 2,
        ..Room::default()
    });
    let grave_lesser_gate = graph.add_node(Room {
        name: "grave_lesser_gate".to_owned(),
        bits: 1,
        cubes: 1,
        ..Room::default()
    });
    let grave_treasure_a = graph.add_node(Room {
        name: "grave_treasure_a".to_owned(),
        bits: 2,
        cubes: 1,
        ..Room::default()
    });
    let graveyard_a = graph.add_node(Room {
        name: "graveyard_a".to_owned(),
        bits: 3,
        ..Room::default()
    });
    let graveyard_gate = graph.add_node(Room {
        name: "graveyard_gate".to_owned(),
        pos: Some((50.0, 50.0)),
        bits: 7,
        ..Room::default()
    });
    let indust_abandoned_a = graph.add_node(Room {
        name: "indust_abandoned_a".to_owned(),
        ..Room::default()
    });
    let industrial_city = graph.add_node(Room {
        name: "industrial_city".to_owned(),
        ..Room::default()
    });
    let industrial_hub = graph.add_node(Room {
        name: "industrial_hub".to_owned(),
        ..Room::default()
    });
    let industrial_superspin = graph.add_node(Room {
        name: "industrial_superspin".to_owned(),
        ..Room::default()
    });
    let kitchen = graph.add_node(Room {
        name: "kitchen".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let lava = graph.add_node(Room {
        name: "lava".to_owned(),
        ..Room::default()
    });
    let lava_fork = graph.add_node(Room {
        name: "lava_fork".to_owned(),
        ..Room::default()
    });
    let lava_skull = graph.add_node(Room {
        name: "lava_skull".to_owned(),
        ..Room::default()
    });
    let library_interior = graph.add_node(Room {
        name: "library_interior".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let lighthouse = graph.add_node(Room {
        name: "lighthouse".to_owned(),
        anti: 1,
        bits: 2,
        ..Room::default()
    });
    let lighthouse_house_a = graph.add_node(Room {
        name: "lighthouse_house_a".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let lighthouse_spin = graph.add_node(Room {
        name: "lighthouse_spin".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let mausoleum = graph.add_node(Room {
        name: "mausoleum".to_owned(),
        bits: 4,
        ..Room::default()
    });
    let memory_core = graph.add_node(Room {
        name: "memory_core".to_owned(),
        pos: Some((25.0, 10.0)),
        ..Room::default()
    });
    let mine_a = graph.add_node(Room {
        name: "mine_a".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let mine_bomb_pillar = graph.add_node(Room {
        name: "mine_bomb_pillar".to_owned(),
        bits: 1,
        keys: 1,
        ..Room::default()
    });
    let mine_wrap = graph.add_node(Room {
        name: "mine_wrap".to_owned(),
        bits: 2,
        cubes: 1,
        ..Room::default()
    });
    let nature_hub = graph.add_node(Room {
        name: "nature_hub".to_owned(),
        pos: Some((25.0, 25.0)),
        bits: 2,
        ..Room::default()
    });
    let nuzu_abandoned_a = graph.add_node(Room {
        name: "nuzu_abandoned_a".to_owned(),
        ..Room::default()
    });
    let nuzu_abandond_b = graph.add_node(Room {
        name: "nuzu_abandond_b".to_owned(),
        ..Room::default()
    });
    let nuzu_boilerroom = graph.add_node(Room {
        name: "nuzu_boilerroom".to_owned(),
        ..Room::default()
    });
    let nuzu_dorm = graph.add_node(Room {
        name: "nuzu_dorm".to_owned(),
        ..Room::default()
    });
    let nuzu_school = graph.add_node(Room {
        name: "nuzu_school".to_owned(),
        ..Room::default()
    });
    let observatory = graph.add_node(Room {
        name: "observatory".to_owned(),
        cubes: 1,
        ..Room::default()
    });
    let oldschool = graph.add_node(Room {
        name: "oldschool".to_owned(),
        ..Room::default()
    });
    let oldschool_ruins = graph.add_node(Room {
        name: "oldschool_ruins".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let orrery = graph.add_node(Room {
        name: "orrery".to_owned(),
        ..Room::default()
    });
    let orrery_b = graph.add_node(Room {
        name: "orrery_b".to_owned(),
        ..Room::default()
    });
    let owl = graph.add_node(Room {
        name: "owl".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let parlor = graph.add_node(Room {
        name: "parlor".to_owned(),
        keys: 1,
        anti: 1,
        ..Room::default()
    });
    let pivot_one = graph.add_node(Room {
        name: "pivot_one".to_owned(),
        ..Room::default()
    });
    let pivot_three = graph.add_node(Room {
        name: "pivot_three".to_owned(),
        ..Room::default()
    });
    let pivot_three_cave = graph.add_node(Room {
        name: "pivot_three_cave".to_owned(),
        ..Room::default()
    });
    let pivot_two = graph.add_node(Room {
        name: "pivot_two".to_owned(),
        ..Room::default()
    });
    let pivot_watertower = graph.add_node(Room {
        name: "pivot_watertower".to_owned(),
        bits: 1,
        anti: 1,
        ..Room::default()
    });
    let purple_lodge = graph.add_node(Room {
        name: "purple_lodge".to_owned(),
        ..Room::default()
    });
    let purple_lodge_ruin = graph.add_node(Room {
        name: "purple_lodge_ruin".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let quantum = graph.add_node(Room {
        name: "quantum".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let rails = graph.add_node(Room {
        name: "rails".to_owned(),
        ..Room::default()
    });
    let ritual = graph.add_node(Room {
        name: "ritual".to_owned(),
        ..Room::default()
    });
    let school = graph.add_node(Room {
        name: "school".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let sewer_fork = graph.add_node(Room {
        name: "sewer_fork".to_owned(),
        ..Room::default()
    });
    let sewer_geyser = graph.add_node(Room {
        name: "sewer_geyser".to_owned(),
        ..Room::default()
    });
    let sewer_hub = graph.add_node(Room {
        name: "sewer_hub".to_owned(),
        ..Room::default()
    });
    let sewer_lesser_gate_b = graph.add_node(Room {
        name: "sewer_lesser_gate_b".to_owned(),
        ..Room::default()
    });
    let sewer_pillars = graph.add_node(Room {
        name: "sewer_pillars".to_owned(),
        ..Room::default()
    });
    let sewer_pivot = graph.add_node(Room {
        name: "sewer_pivot".to_owned(),
        ..Room::default()
    });
    let sewer_qr = graph.add_node(Room {
        name: "sewer_qr".to_owned(),
        ..Room::default()
    });
    let sewer_start = graph.add_node(Room {
        name: "sewer_start".to_owned(),
        ..Room::default()
    });
    let sewer_to_lava = graph.add_node(Room {
        name: "sewer_to_lava".to_owned(),
        ..Room::default()
    });
    let sewer_treasure_one = graph.add_node(Room {
        name: "sewer_treasure_one".to_owned(),
        ..Room::default()
    });
    let sewer_treasure_two = graph.add_node(Room {
        name: "sewer_treasure_two".to_owned(),
        ..Room::default()
    });
    let showers = graph.add_node(Room {
        name: "showers".to_owned(),
        ..Room::default()
    });
    let skull = graph.add_node(Room {
        name: "skull".to_owned(),
        bits: 3,
        ..Room::default()
    });
    let skull_b = graph.add_node(Room {
        name: "skull_b".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let spinning_plates = graph.add_node(Room {
        name: "spinning_plates".to_owned(),
        ..Room::default()
    });
    let stargate = graph.add_node(Room {
        name: "stargate".to_owned(),
        ..Room::default()
    });
    let stargate_ruins = graph.add_node(Room {
        name: "stargate_ruins".to_owned(),
        bits: 2,
        ..Room::default()
    });
    let superspin_cave = graph.add_node(Room {
        name: "superspin_cave".to_owned(),
        ..Room::default()
    });
    let telescope = graph.add_node(Room {
        name: "telescope".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let temple_of_love = graph.add_node(Room {
        name: "temple_of_love".to_owned(),
        ..Room::default()
    });
    let throne = graph.add_node(Room {
        name: "throne".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let tree = graph.add_node(Room {
        name: "tree".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let tree_crumble = graph.add_node(Room {
        name: "tree_crumble".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let tree_of_death = graph.add_node(Room {
        name: "tree_of_death".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let tree_roots = graph.add_node(Room {
        name: "tree_roots".to_owned(),
        ..Room::default()
    });
    let tree_sky = graph.add_node(Room {
        name: "tree_sky".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let triple_pivot_cave = graph.add_node(Room {
        name: "triple_pivot_cave".to_owned(),
        ..Room::default()
    });
    let two_walls = graph.add_node(Room {
        name: "two_walls".to_owned(),
        bits: 3,
        cubes: 1,
        ..Room::default()
    });
    let villageville_3d = graph.add_node(Room {
        name: "villageville_3d".to_owned(),
        alias: "villageville".to_owned(),
        keys: 1,
        bits: 4,
        ..Room::default()
    });
    // doesn't exist?
    let village_exit = graph.add_node(Room {
        name: "village_exit".to_owned(),
        ..Room::default()
    });
    let visitor = graph.add_node(Room {
        name: "visitor".to_owned(),
        cubes: 1,
        ..Room::default()
    });
    // let wall_a = graph.add_node(Room {
    //     name: "wall_a".to_owned(),
    //     ..Room::default()
    // });
    // let wall_b = graph.add_node(Room {
    //     name: "wall_b".to_owned(),
    //     ..Room::default()
    // });
    let wall_hole = graph.add_node(Room {
        name: "wall_hole".to_owned(),
        bits: 2,
        keys: 1,
        ..Room::default()
    });
    let wall_interior_a = graph.add_node(Room {
        name: "wall_interior_a".to_owned(),
        ..Room::default()
    });
    let wall_interior_b = graph.add_node(Room {
        name: "wall_interior_b".to_owned(),
        ..Room::default()
    });
    let wall_interior_hole = graph.add_node(Room {
        name: "wall_interior_hole".to_owned(),
        ..Room::default()
    });
    let wall_kitchen = graph.add_node(Room {
        name: "wall_kitchen".to_owned(),
        ..Room::default()
    });
    let wall_school = graph.add_node(Room {
        name: "wall_school".to_owned(),
        ..Room::default()
    });
    let wall_village = graph.add_node(Room {
        name: "wall_village".to_owned(),
        ..Room::default()
    });
    let water_pyramid = graph.add_node(Room {
        name: "water_pyramid".to_owned(),
        ..Room::default()
    });
    let water_tower = graph.add_node(Room {
        name: "water_tower".to_owned(),
        ..Room::default()
    });
    let water_wheel = graph.add_node(Room {
        name: "water_wheel".to_owned(),
        ..Room::default()
    });
    let water_wheel_b = graph.add_node(Room {
        name: "water_wheel_b".to_owned(),
        ..Room::default()
    });
    let waterfall = graph.add_node(Room {
        name: "waterfall".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let watertower_secret = graph.add_node(Room {
        name: "watertower_secret".to_owned(),
        ..Room::default()
    });
    let weightswitch_temple = graph.add_node(Room {
        name: "weightswitch_temple".to_owned(),
        bits: 2,
        ..Room::default()
    });
    let well_2 = graph.add_node(Room {
        name: "well_2".to_owned(),
        ..Room::default()
    });
    let windmill_cave = graph.add_node(Room {
        name: "windmill_cave".to_owned(),
        ..Room::default()
    });
    let windmill_int = graph.add_node(Room {
        name: "windmill_int".to_owned(),
        ..Room::default()
    });
    let zu_4_side = graph.add_node(Room {
        name: "zu_4_side".to_owned(),
        alias: "zu_four_side".to_owned(),
        ..Room::default()
    });
    let zu_bridge = graph.add_node(Room {
        name: "zu_bridge".to_owned(),
        bits: 2,
        anti: 1,
        ..Room::default()
    });
    let zu_city = graph.add_node(Room {
        name: "zu_city".to_owned(),
        ..Room::default()
    });
    let zu_city_ruins = graph.add_node(Room {
        name: "zu_city_ruins".to_owned(),
        pos: Some((0.0, 50.0)),
        ..Room::default()
    });
    let zu_code_loop = graph.add_node(Room {
        name: "zu_code_loop".to_owned(),
        bits: 2,
        cubes: 1,
        anti: 1,
        ..Room::default()
    });
    let zu_fork = graph.add_node(Room {
        name: "zu_fork".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let zu_heads = graph.add_node(Room {
        name: "zu_heads".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let zu_house_empty = graph.add_node(Room {
        name: "zu_house_empty".to_owned(),
        ..Room::default()
    });
    let zu_house_empty_b = graph.add_node(Room {
        name: "zu_house_empty_b".to_owned(),
        ..Room::default()
    });
    let zu_house_qr = graph.add_node(Room {
        name: "zu_house_qr".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let zu_house_ruin_gate = graph.add_node(Room {
        name: "zu_house_ruin_gate".to_owned(),
        ..Room::default()
    });
    let zu_house_ruin_visitors = graph.add_node(Room {
        name: "zu_house_ruin_visitors".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let zu_house_scaffolding = graph.add_node(Room {
        name: "zu_house_scaffolding".to_owned(),
        ..Room::default()
    });
    let zu_library = graph.add_node(Room {
        name: "zu_library".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let zu_switch = graph.add_node(Room {
        name: "zu_switch".to_owned(),
        ..Room::default()
    });
    let zu_switch_b = graph.add_node(Room {
        name: "zu_switch_b".to_owned(),
        cubes: 1,
        ..Room::default()
    });
    let zu_tetris = graph.add_node(Room {
        name: "zu_tetris".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let zu_throne_ruins = graph.add_node(Room {
        name: "zu_throne_ruins".to_owned(),
        bits: 1,
        ..Room::default()
    });
    let zu_unfold = graph.add_node(Room {
        name: "zu_unfold".to_owned(),
        anti: 1,
        ..Room::default()
    });
    let zu_zuish = graph.add_node(Room {
        name: "zu_zuish".to_owned(),
        ..Room::default()
    });

    graph.add_edge(villageville_3d, abandoned_a, Door);
    graph.add_edge(villageville_3d, abandoned_b, Door);
    graph.add_edge(villageville_3d, abandoned_c, Door);
    graph.add_edge(villageville_3d, boileroom, Lock);
    graph.add_edge(villageville_3d, geezer_house, Door);
    graph.add_edge(villageville_3d, gomez_house, Door);
    graph.add_edge(villageville_3d, kitchen, Door);
    graph.add_edge(villageville_3d, parlor, Door);
    graph.add_edge(villageville_3d, school, Door);
    graph.add_edge(villageville_3d, big_tower, Door);
    graph.add_edge(big_tower, village_exit, Door);
    graph.add_edge(village_exit, memory_core, Door);

    graph.add_edge(memory_core, industrial_city, Door);
    graph.add_edge(memory_core, wall_village, Door);
    graph.add_edge(memory_core, zu_city, Door);
    graph.add_edge(memory_core, nature_hub, Door);

    graph.add_edge(industrial_city, nuzu_abandoned_a, Door);
    graph.add_edge(industrial_city, nuzu_boilerroom, Door);
    graph.add_edge(industrial_city, nuzu_dorm, Door);
    graph.add_edge(industrial_city, nuzu_school, Door);
    graph.add_edge(industrial_city, showers, Door);

    graph.add_edge(wall_village, wall_interior_a, Door);
    graph.add_edge(wall_village, wall_interior_b, Door);
    graph.add_edge(wall_village, wall_interior_hole, Door);
    graph.add_edge(wall_village, wall_kitchen, Door);
    graph.add_edge(wall_village, wall_school, Door);

    graph.add_edge(zu_city, oldschool, Door);
    graph.add_edge(zu_city, purple_lodge, Door);
    graph.add_edge(zu_city, stargate, Door);
    graph.add_edge(zu_city, zu_house_empty, Door);
    graph.add_edge(zu_city, zu_house_empty_b, Door);
    graph.add_edge(zu_city, zu_house_ruin_gate, Door);
    graph.add_edge(zu_city, zu_house_scaffolding, Door);

    graph.add_edge(nature_hub, ritual, Water);
    graph.add_edge(nature_hub, arch, Door);
    graph.add_edge(arch, weightswitch_temple, Door);
    graph.add_edge(weightswitch_temple, zu_switch, Door);
    graph.add_edge(zu_switch, zu_switch_b, Door);
    graph.add_edge(zu_switch_b, nature_hub, Warp);
    graph.add_edge(arch, five_towers, Door);
    graph.add_edge(five_towers, five_towers_cave, Door);
    graph.add_edge(five_towers, bell_tower, SecretBi);
    graph.add_edge(five_towers, nature_hub, Warp);
    graph.add_edge(nature_hub, waterfall, Door);
    graph.add_edge(nature_hub, bell_tower, Door);
    graph.add_edge(nature_hub, tree_roots, Door);
    graph.add_edge(nature_hub, lighthouse, Door);

    graph.add_edge(waterfall, cmy, Door);
    graph.add_edge(cmy, cmy_fork, Door);
    graph.add_edge(cmy, cmy_b, Door);
    graph.add_edge(cmy_b, nature_hub, Warp);
    graph.add_edge(waterfall, mine_a, Door);
    graph.add_edge(mine_a, mine_wrap, Door);
    graph.add_edge(mine_wrap, nature_hub, Warp);
    graph.add_edge(mine_wrap, mine_bomb_pillar, Door);
    graph.add_edge(waterfall, zu_code_loop, Door);
    graph.add_edge(zu_code_loop, nature_hub, Warp);
    graph.add_edge(waterfall, fox, Door);
    graph.add_edge(waterfall, water_wheel, Door);
    graph.add_edge(water_wheel, water_wheel_b, Door);
    graph.add_edge(waterfall, zu_zuish, Water);

    graph.add_edge(bell_tower, water_pyramid, Door);
    graph.add_edge(water_pyramid, temple_of_love, Door);
    graph.add_edge(bell_tower, quantum, Water);
    graph.add_edge(quantum, nature_hub, Warp);
    graph.add_edge(bell_tower, ancient_walls, Door);
    graph.add_edge(ancient_walls, zu_tetris, Door);
    graph.add_edge(ancient_walls, wall_hole, Door);
    graph.add_edge(wall_hole, two_walls, Door);
    graph.add_edge(two_walls, nature_hub, Secret);
    graph.add_edge(two_walls, nature_hub, Warp);
    graph.add_edge(two_walls, fractal, Door);
    graph.add_edge(fractal, zu_4_side, Door);
    graph.add_edge(zu_4_side, zu_heads, Door);
    // TODO what are these rooms actually called? they didn't show up in bell_tower.json
    // graph.add_edge(bell_tower, wall_a, Door);
    // graph.add_edge(wall_a, wall_b, Door);
    // graph.add_edge(wall_b, nature_hub, Warp);

    graph.add_edge(tree_roots, tree, Door);
    graph.add_edge(tree, cabin_interior, Door);
    graph.add_edge(cabin_interior, grave_cabin, Door);
    graph.add_edge(grave_cabin, graveyard_gate, Door);
    graph.add_edge(tree, tree_crumble, Lock);
    graph.add_edge(tree, tree_sky, Door);
    graph.add_edge(tree_sky, throne, Door);
    graph.add_edge(throne, zu_unfold, Door);
    graph.add_edge(throne, zu_bridge, Door);
    graph.add_edge(zu_bridge, zu_city_ruins, Door);

    graph.add_edge(lighthouse, zu_fork, Water);
    graph.add_edge(lighthouse, lighthouse_house_a, Lock);
    graph.add_edge(lighthouse_house_a, lighthouse_spin, Door);
    graph.add_edge(lighthouse_spin, lighthouse, Door);
    graph.add_edge(lighthouse, water_tower, Door);
    graph.add_edge(water_tower, watertower_secret, Door);
    graph.add_edge(water_tower, pivot_watertower, Door);
    graph.add_edge(pivot_watertower, industrial_hub, Door);
    graph.add_edge(pivot_watertower, memory_core, Secret);

    graph.add_edge(graveyard_gate, nature_hub, Warp);
    graph.add_edge(graveyard_gate, owl, Door);
    graph.add_edge(owl, big_owl, Owl);
    graph.add_edge(grave_lesser_gate, skull, Door);
    graph.add_edge(skull, skull_b, Door);
    graph.add_edge(skull_b, graveyard_gate, Warp);
    graph.add_edge(graveyard_gate, mausoleum, Door);
    graph.add_edge(mausoleum, tree_roots, Secret);
    graph.add_edge(mausoleum, grave_ghost, Door);
    graph.add_edge(mausoleum, crypt, Lock);
    graph.add_edge(crypt, tree_of_death, Door);
    graph.add_edge(mausoleum, graveyard_a, Door);
    graph.add_edge(graveyard_a, grave_lesser_gate, Door);
    graph.add_edge(grave_lesser_gate, graveyard_gate, Warp);
    graph.add_edge(graveyard_a, grave_treasure_a, Door);
    graph.add_edge(grave_treasure_a, graveyard_gate, Warp);

    graph.add_edge(zu_city_ruins, nature_hub, Warp);
    graph.add_edge(zu_city_ruins, zu_house_qr, Door);
    graph.add_edge(zu_city_ruins, oldschool_ruins, Door);
    graph.add_edge(zu_city_ruins, purple_lodge_ruin, Door);
    graph.add_edge(zu_city_ruins, zu_house_ruin_visitors, Door);
    graph.add_edge(zu_city_ruins, zu_throne_ruins, Door);
    graph.add_edge(zu_city_ruins, stargate_ruins, Door);
    graph.add_edge(zu_city_ruins, clock, Door);
    graph.add_edge(clock, zu_library, Door);
    graph.add_edge(clock, zu_city_ruins, Warp);
    graph.add_edge(zu_library, library_interior, Door);
    graph.add_edge(zu_library, zu_city_ruins, Secret);
    graph.add_edge(library_interior, globe, Door);
    graph.add_edge(globe, globe_int, Door);
    graph.add_edge(zu_city_ruins, observatory, Door);
    graph.add_edge(observatory, throne, Secret);
    graph.add_edge(observatory, telescope, Door);
    graph.add_edge(observatory, visitor, Door);
    graph.add_edge(visitor, purple_lodge_ruin, SecretIndirect);
    graph.add_edge(visitor, zu_city_ruins, Warp);
    graph.add_edge(visitor, code_machine, Door);
    graph.add_edge(visitor, orrery, Door);
    graph.add_edge(orrery, orrery_b, Door);

    graph.add_edge(industrial_hub, nature_hub, Warp);
    graph.add_edge(industrial_hub, indust_abandoned_a, Door);
    graph.add_edge(industrial_hub, nuzu_abandond_b, Door);
    graph.add_edge(industrial_hub, industrial_superspin, Door);
    graph.add_edge(industrial_superspin, superspin_cave, Door);
    graph.add_edge(superspin_cave, industrial_hub, Warp);
    graph.add_edge(industrial_hub, pivot_one, Door);
    graph.add_edge(pivot_one, windmill_int, Door);
    graph.add_edge(windmill_int, windmill_cave, Door);
    graph.add_edge(pivot_one, pivot_two, Door);
    graph.add_edge(pivot_two, extractor_a, Door);
    graph.add_edge(pivot_two, pivot_three, Door);
    graph.add_edge(pivot_three, industrial_hub, Warp);
    graph.add_edge(pivot_three, pivot_three_cave, Door);
    graph.add_edge(industrial_hub, rails, Door);
    graph.add_edge(rails, triple_pivot_cave, Door);
    graph.add_edge(triple_pivot_cave, spinning_plates, Door);
    graph.add_edge(spinning_plates, industrial_hub, Warp);
    graph.add_edge(triple_pivot_cave, well_2, Door);
    graph.add_edge(well_2, industrial_hub, SecretBi);
    graph.add_edge(well_2, sewer_start, Door);
    graph.add_edge(sewer_start, sewer_hub, Door);

    graph.add_edge(sewer_hub, industrial_hub, Warp);
    graph.add_edge(sewer_hub, nature_hub, Warp);
    graph.add_edge(sewer_hub, sewer_treasure_one, Door);
    graph.add_edge(sewer_hub, sewer_pivot, Door);
    graph.add_edge(sewer_hub, sewer_qr, Door);
    graph.add_edge(sewer_hub, sewer_to_lava, Door);
    graph.add_edge(sewer_to_lava, nuzu_abandond_b, SecretIndirect);
    graph.add_edge(sewer_to_lava, lava, Door);
    graph.add_edge(lava, lava_skull, Door);
    graph.add_edge(lava_skull, sewer_hub, Warp);
    graph.add_edge(lava_skull, lava_fork, Door);
    graph.add_edge(sewer_hub, sewer_geyser, Door);
    graph.add_edge(sewer_geyser, sewer_pillars, Door);
    graph.add_edge(sewer_pillars, sewer_treasure_two, Door);
    graph.add_edge(sewer_pillars, sewer_fork, Door);
    graph.add_edge(sewer_fork, sewer_hub, Secret);
    graph.add_edge(sewer_pillars, sewer_lesser_gate_b, Door);
    graph.add_edge(sewer_lesser_gate_b, sewer_hub, Warp);

    graph
}

pub fn render(path: impl AsRef<Path>) {
    if let Err(e) = try_render(path.as_ref()) {
        error!("failed to generate graphviz at {:?}: {}", path.as_ref(), e);
    }
}
fn try_render(path: &Path) -> io::Result<()> {
    let graph = rooms();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = fs::File::create(path)?;

    writeln!(output, "strict digraph {{")?;
    writeln!(output, "  graph [bgcolor=black K=1]")?;

    graph.node_references().try_for_each(|(i, n)| {
        let size = graph.edges_directed(i, Incoming).count() +
        graph.edges_directed(i, Outgoing).count();
        let size = (size as f64).sqrt() * 0.8;

        write!(
            output,
            "  \"{}\" [color=white fontcolor=white shape=box fixedsize=true width={} height={1} label=\"{}\"",
            n.name,
            size,
            n.name.replace("_", " ")
        )?;
        if let Some((x, y)) = n.pos {
            write!(
                output, " pos=\"{},{}\" pin=true",
                x, y
            )?;
        }
        writeln!(output, "]")
    })?;

    graph
        .edge_references()
        .map(|e| (&graph[e.source()].name, &graph[e.target()].name, e.weight()))
        .try_for_each(|(s, t, e)| match e {
            Door::Door | Door::Water | Door::Owl => {
                writeln!(
                    output,
                    "  \"{}\" -> \"{}\" [dir=both color=white len=1]",
                    s, t
                )
            }
            Door::Lock => writeln!(output, "  \"{}\" -> \"{}\" [color=red len=1]", s, t),
            // flipped to handle two_walls which has two edges to nature_hub
            Door::Secret => writeln!(
                output,
                "  \"{}\" -> \"{}\" [dir=back color=yellow len=5 weight=0]",
                t, s
            ),
            Door::SecretIndirect => {
                writeln!(
                    output,
                    "  \"{}\" -> \"{}\" [color=yellow len=5 weight=0]",
                    s, t
                )
            }
            Door::SecretBi => writeln!(
                output,
                "  \"{}\" -> \"{}\" [dir=both color=yellow len=5 weight=0]",
                s, t
            ),
            Door::Warp => writeln!(
                output,
                "  \"{}\" -> \"{}\" [color=blue len=5 weight=0]",
                s, t
            ),
        })?;

    writeln!(output, "}}")?;

    Ok(())
}
