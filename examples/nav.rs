// In this game, the player navigates to wherever you click

use bevy::{prelude::*, sprite::Anchor};
use rand::{thread_rng, Rng};
use seldom_map_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin is required for pathfinding and navigation
        // The type parameter is the position component that you use
        .add_plugin(MapNavPlugin::<Transform>::default())
        .init_resource::<CursorPos>()
        .add_startup_system(init)
        .add_systems((update_cursor_pos, move_player).chain())
        .run();
}

const MAP_SIZE: UVec2 = UVec2::new(24, 24);
const TILE_SIZE: Vec2 = Vec2::new(32., 32.);
// This is the radius of a square around the player that should not intersect with the terrain
const PLAYER_CLEARANCE: f32 = 8.;

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        // Centering the camera
        transform: Transform::from_translation((MAP_SIZE.as_vec2() * TILE_SIZE / 2.).extend(999.9)),
        ..default()
    });

    let mut rng = thread_rng();
    // Randomly generate the tilemap
    let tilemap = [(); (MAP_SIZE.x * MAP_SIZE.y) as usize].map(|_| match rng.gen_bool(0.8) {
        true => Navability::Navable,
        false => Navability::Solid,
    });
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_SIZE.x + pos.x) as usize];

    // Spawn images for the tiles
    let tile_image = asset_server.load("tile.png");
    let mut player_pos = default();
    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            let pos = UVec2::new(x, y);
            if let Navability::Navable = navability(pos) {
                let pos = UVec2::new(x, y).as_vec2() * TILE_SIZE;
                player_pos = pos;

                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.)),
                    texture: tile_image.clone(),
                    ..default()
                });
            }
        }
    }

    // Here's the important bit:

    // Spawn the tilemap with a `Navmeshes` component
    commands
        .spawn(Navmeshes::generate(MAP_SIZE, TILE_SIZE, navability, [PLAYER_CLEARANCE]).unwrap());

    // Spawn the player component. A position component is necessary. We will add `NavBundle`
    // later.
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation((player_pos + TILE_SIZE / 2.).extend(1.)),
            texture: asset_server.load("player.png"),
            ..default()
        },
        Player,
    ));
}

// Navigate the player to wherever you click
fn move_player(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    navmesheses: Query<Entity, With<Navmeshes>>,
    cursor_pos: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = **cursor_pos {
            // Clicked somewhere on the screen!
            // Add `NavBundle` to start navigating to that position
            // If you want to write your own movement, but still want paths generated,
            // only insert `Pathfind`.
            commands.entity(players.single()).insert(NavBundle {
                pathfind: Pathfind::new(
                    navmesheses.single(),
                    PLAYER_CLEARANCE,
                    None,
                    PathTarget::Static(cursor_pos),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                ),
                nav: Nav::new(200.),
            });
        }
    }
}

// The code after this comment is not related to `seldom_map_nav`

#[derive(Component)]
struct Player;

#[derive(Default, Deref, DerefMut, Resource)]
struct CursorPos(Option<Vec2>);

fn update_cursor_pos(
    cameras: Query<&Transform, With<Camera2d>>,
    windows: Query<&Window>,
    mut pos: ResMut<CursorPos>,
) {
    let window = windows.single();
    **pos = window.cursor_position().map(|cursor_pos| {
        (cameras.single().compute_matrix()
            * (cursor_pos - Vec2::new(window.width(), window.height()) / 2.)
                .extend(0.)
                .extend(1.))
        .truncate()
        .truncate()
    });
}
