// In this game, the player navigates to wherever you click on an isometric map.

// The tilemap displays as isometric, but the navmesh is square, so we use square-to-iso conversions
// and vice versa. To convert from square to iso coordinates, we:
// 1. Rotate them by 45 degrees clockwise
// 2. Scale Y by 0.5
// Converting iso to square coordinates is the inverse: first we scale Y by 2, and then rotate it
// counter-clockwise.

// To separate these coordinate systems, we use a custom position component, `SquarePos`.

use std::f32::consts::FRAC_1_SQRT_2;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};
use seldom_map_nav::prelude::*;

// Custom position component for navigation
#[derive(Clone, Component, Copy)]
struct SquarePos(Vec2);

// Required to nav with this component
impl Position2 for SquarePos {
    fn get(&self) -> Vec2 {
        let &Self(square) = self;
        square
    }

    fn set(&mut self, pos: Vec2) {
        let Self(square) = self;
        *square = pos;
    }
}

impl SquarePos {
    fn from_iso(mut iso: Vec2) -> Self {
        // FRAC_1_SQRT_2 = 1 / sqrt(2) = sin(45) = cos(45)
        iso.y *= 2.;
        Self(Vec2 {
            x: iso.x * FRAC_1_SQRT_2 - iso.y * FRAC_1_SQRT_2,
            y: iso.x * FRAC_1_SQRT_2 + iso.y * FRAC_1_SQRT_2,
        })
    }

    fn to_iso(self) -> Vec2 {
        let Self(square) = self;
        Vec2 {
            x: square.x * FRAC_1_SQRT_2 + square.y * FRAC_1_SQRT_2,
            y: (-square.x * FRAC_1_SQRT_2 + square.y * FRAC_1_SQRT_2) / 2.,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // This plugin is required for pathfinding and navigation. The type parameter is the
            // position component that you use. We use `SquarePos` instead of `Transform`.
            MapNavPlugin::<SquarePos>::default(),
            TilemapPlugin,
        ))
        .init_resource::<CursorPos>()
        .add_systems(Startup, init)
        .add_systems(
            Update,
            (update_cursor_pos, move_player, translate_coords).chain(),
        )
        .run();
}

const MAP_SIZE: UVec2 = UVec2::new(12, 12);
const TILE_SIZE: Vec2 = Vec2::new(100., 50.);

// add 0.5X offset because our visible tiles spawned with center anchor
const MAP_OFFSET: Vec2 = Vec2::new(50., 0.);

// This is the radius of a square around the player that should not intersect with the terrain
const PLAYER_CLEARANCE: f32 = 8.;

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        // Center the camera
        transform: Transform::from_translation(Vec3::new(
            MAP_SIZE.x as f32 * TILE_SIZE.x / 2.,
            0.,
            0.,
        )),
        ..default()
    });

    let mut rng = thread_rng();
    // Randomly generate the tilemap
    let tilemap = [(); (MAP_SIZE.x * MAP_SIZE.y) as usize].map(|_| match rng.gen_bool(0.8) {
        true => Navability::Navable,
        false => Navability::Solid,
    });
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_SIZE.x + pos.x) as usize];

    // Setup `bevy_ecs_tilemap`
    let map_size = MAP_SIZE.into();
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    // To get the square tile size that fits our iso coords, we need to take our iso tile, scale it
    // up by a factor of 2 along the Y-axis, and then calculate the length of the side of the
    // resulting tile. This requires applying the Pythagorean theorem to a triangle with sides of
    // length X/2 and X/2, where X is the size of our iso tile along the X-axis. So, the square side
    // length is given by sqrt(x^2 / 2).
    let navmap_tile_size = Vec2::splat((TILE_SIZE.x * TILE_SIZE.x / 2.).sqrt());

    // Spawn images for the tiles
    let tile_image = asset_server.load("tile-iso.png");
    let mut player_pos = default();

    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            let pos = UVec2::new(x, y);
            if let Navability::Navable = navability(pos) {
                player_pos = pos.as_vec2() * navmap_tile_size;

                // Spawning tiles
                let tile_pos = pos.into();
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index: TileTextureIndex(0),
                        ..default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TILE_SIZE.into(),
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_image),
        tile_size: TILE_SIZE.into(),
        map_type: TilemapType::Isometric(IsoCoordSystem::Diamond),
        transform: Transform::from_xyz(MAP_OFFSET.x, MAP_OFFSET.y, 0.),
        ..default()
    });

    // Here's the important bit:

    // Spawn the tilemap with a `Navmeshes` component
    commands.spawn(
        Navmeshes::generate(MAP_SIZE, navmap_tile_size, navability, [PLAYER_CLEARANCE]).unwrap(),
    );

    // Spawn the player component. A position component is necessary. We will add `NavBundle` later.
    let square_pos = SquarePos(player_pos + TILE_SIZE / 2.);
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(square_pos.to_iso().extend(1.)),
            texture: asset_server.load("player.png"),
            ..default()
        },
        Player,
        square_pos,
    ));
}

#[derive(Default, Deref, DerefMut, Resource)]
struct CursorPos(Option<Vec2>);

fn update_cursor_pos(
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut position: ResMut<CursorPos>,
) {
    let (camera, transform) = cameras.single();
    **position = windows
        .single()
        .cursor_position()
        .and_then(|cursor_pos| camera.viewport_to_world_2d(transform, cursor_pos));
}

#[derive(Component)]
struct Player;

// Navigate the player to wherever you click
fn move_player(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    navmesheses: Query<Entity, With<Navmeshes>>,
    cursor_pos: Res<CursorPos>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = **cursor_pos {
            // Clicked somewhere on the screen!
            // Add `NavBundle` to start navigating to that position
            // If you want to write your own movement, but still want paths generated, only insert
            // `Pathfind`.
            let SquarePos(square_coord) = SquarePos::from_iso(cursor_pos);
            commands.entity(players.single()).insert(NavBundle {
                pathfind: Pathfind::new(
                    navmesheses.single(),
                    PLAYER_CLEARANCE,
                    None,
                    PathTarget::Static(square_coord),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                ),
                nav: Nav::new(200.),
            });
        }
    }
}

// Update isometric positions to match square positions
fn translate_coords(mut players: Query<(&mut Transform, &SquarePos)>) {
    let (mut transform, square) = players.single_mut();
    let iso = square.to_iso();
    transform.translation.x = iso.x;
    transform.translation.y = iso.y;
}
