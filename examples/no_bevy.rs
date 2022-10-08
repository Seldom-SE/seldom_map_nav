// In this program, we generate paths through a tilemap

use glam::{UVec2, Vec2};
use navmesh::NavVec3;
use seldom_map_nav::{mesh::generate_navmesh, prelude::*};

// For simplicity, let's assume 0, 0 is at the top left
#[rustfmt::skip]
static TILEMAP: &[&[bool]] = &[
    &[true,  true,  true,  true,  false],
    &[true,  false, false, true,  true],
    &[true,  true,  true,  false, true],
    &[true,  true,  false, false, true],
];
const MAP_SIZE: UVec2 = UVec2::new(5, 4);
const TILE_SIZE: Vec2 = Vec2::splat(1.);
const START: NavVec3 = NavVec3 {
    x: 1.5,
    y: 3.5,
    z: 0.,
};
const END: NavVec3 = NavVec3 {
    x: 4.5,
    y: 3.5,
    z: 0.,
};

fn main() {
    let navability = |position: UVec2| match TILEMAP[position.y as usize][position.x as usize] {
        true => Navability::Navable,
        false => Navability::Solid,
    };

    // Either generate a single navmesh with `seldom_map_nav::mesh::generate_navmesh`...

    // Print the path with 0.2 clearance
    println!(
        "{:?}",
        generate_navmesh(MAP_SIZE, TILE_SIZE, navability, 0.2)
            .unwrap()
            .find_path(START, END, NavQuery::Accuracy, NavPathMode::Accuracy)
            .unwrap()
    );

    // ...Or, if you have multiple navigators of different sizes, generate multiple, using `Navmeshes::generate`

    // Generate three navmeshes, each with different clearance values
    let navmeshes = Navmeshes::generate(MAP_SIZE, TILE_SIZE, navability, [0., 0.2, 0.4]).unwrap();

    // Print a path with at least 0. clearance (0. in this case)
    println!(
        "{:?}",
        navmeshes
            .mesh(0.)
            .unwrap()
            .find_path(START, END, NavQuery::Accuracy, NavPathMode::Accuracy)
            .unwrap()
    );

    // Print a path with at least 0.3 clearance (0.4 in this case)
    println!(
        "{:?}",
        navmeshes
            .mesh(0.3)
            .unwrap()
            .find_path(START, END, NavQuery::Accuracy, NavPathMode::Accuracy)
            .unwrap()
    );
}
