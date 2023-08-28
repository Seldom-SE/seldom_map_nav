/* In this game, the player navigates to wherever you click.
   Map is isometric, but navmesh is still squared, so everything works through square-to-iso and backwards conversions.
   My idea for converting squared XY to iso XY:
   1. Get squared XY coords in Vec2
   2. Rotate them by 45 degree clockwise
   3. Scale result to 0.5 by Y (standard size for iso)
   You may notice that this is the way to convert tile to iso tile, and it works for whole map or map position the same way.
   Iso XY to square XY works the opposite way - first we scale Y by 2., and then rotate it counter-clockwise.

   In this example, we initializing MapNavPlugin with custom Transform2 type instead of transform.
   Plugin will "move" our player in squared navmesh, updating squared coords and writing them to Transform2 component.
   We can convert them to iso coords and update Transform of player during runtime.
 */

// run with cargo run --example isometric_map --features="iso_map"
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use seldom_map_nav::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use seldom_interop::prelude::Position2;


// custom type to get square XY from MapNavPlugin
#[derive(Component)]
pub struct Transform2(pub Vec2);

impl Position2 for Transform2 {
    type Position = Vec2;

    fn get(&self) -> Self::Position { self.0 }
    fn set(&mut self, pos: Self::Position) { self.0 = pos }
}

fn nav_to_iso_coord(v: Vec2) -> Vec2 {
    let angle_rad = -45_f32.to_radians();
    let new_x = v.x * angle_rad.cos() - v.y * angle_rad.sin();
    let new_y = v.x * angle_rad.sin() + v.y * angle_rad.cos();
    Vec2 { x: new_x, y: new_y * 0.5}
}

fn iso_to_nav_coord(v: Vec2) -> Vec2 {
    let angle_rad = 45_f32.to_radians();
    let new_x = v.x * angle_rad.cos() - v.y * angle_rad.sin() * 2.;
    let new_y = v.x * angle_rad.sin() + v.y * angle_rad.cos() * 2.;
    Vec2 { x: new_x, y: new_y }
}

fn main() {
    App::new()
        // This plugin is required for pathfinding and navigation.
        // The type parameter is the position component that you use. We using Transform2 instead of Transform
        .add_plugins((DefaultPlugins, MapNavPlugin::<Transform2>::default()))

        .add_plugins(TilemapPlugin)
        .init_resource::<CursorPos>()
        .add_systems(Startup, init)
        .add_systems(Update, (update_cursor_pos, move_player, translate_coords).chain())
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
        // Centering the camera
        transform: Transform::from_translation(Vec3::new(MAP_SIZE.x as f32 * TILE_SIZE.x / 2., 0., 0.)),
        ..default()
    });

    let mut rng = thread_rng();
    // Randomly generate the tilemap
    let tilemap = [(); (MAP_SIZE.x * MAP_SIZE.y) as usize].map(|_| match rng.gen_bool(0.8) {
        true => Navability::Navable,
        false => Navability::Solid,
    });
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_SIZE.x + pos.x) as usize];

    // init ecs_tilemap stuff
    let map_size = MAP_SIZE.into();
    let tile_size = TILE_SIZE.into();
    let grid_size = TILE_SIZE.into();
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    //  To get the squared tile size which will fit our iso coords, we should get our iso tile, upscale it twice by Y,
    // and then calculate the length of drawn tile's side, which requires to apply Pythagoras theorem for triangle
    // with sides X/2 and X/2, where X is our iso tile size by X:
    //  squared tile size we are looking for = sqrt(x^2 / 2)
    let navmap_tile_size = f32::sqrt(TILE_SIZE.x * TILE_SIZE.x / 2.);
    let navmap_tile_size = Vec2::new(navmap_tile_size, navmap_tile_size);

    // Spawn images for the tiles
    let tile_image = asset_server.load("tile-iso.png");
    let mut player_pos = default();

    // Good practice is to attach tiles to single parent for easier access
    commands.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..MAP_SIZE.x {
            for y in 0..MAP_SIZE.y {
                let pos = UVec2::new(x, y);
                if let Navability::Navable = navability(pos) {
                    // updating player squared pos, with unoptimal but explicit way
                    player_pos = UVec2::new(x, y).as_vec2() * navmap_tile_size;

                    // spawning tiles, they remain invisible until we spawn tilemap
                    let tile_pos = pos.into();
                    let tile_entity = parent.spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index: TileTextureIndex(0),
                        ..Default::default()
                    }).id();
                    tile_storage.set(&tile_pos, tile_entity);
                }
            }
        }
    });

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_image),
        tile_size,
        map_type: TilemapType::Isometric(IsoCoordSystem::Diamond),
        // transform: get_tilemap_center_transform(&map_size, &grid_size, & TilemapType::Isometric(IsoCoordSystem::Diamond), 0.0),
        transform: Transform::from_xyz(MAP_OFFSET.x, MAP_OFFSET.y, 0.),
        ..Default::default()
    });

    // Here's the important bit:

    // Spawn the tilemap with a `Navmeshes` component
    commands
        .spawn(Navmeshes::generate(MAP_SIZE, navmap_tile_size, navability, [PLAYER_CLEARANCE]).unwrap());

    // Spawn the player component. A position component is necessary. We will add `NavBundle`
    // later.
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(nav_to_iso_coord(player_pos + TILE_SIZE / 2.).extend(1.)),
            texture: asset_server.load("player.png"),
            ..default()
        },
        Player,
        Transform2(Vec2::ZERO)
    ));
}

fn translate_coords(
    // get coords from navPlugin, translate them to iso, update player position
    mut players: Query<(&mut Transform, &Transform2), With<Player>>,
) {
    let (mut transform, transform2) = players.single_mut();
    let iso_pos = nav_to_iso_coord(transform2.0);
    transform.translation.x = iso_pos.x;
    transform.translation.y = iso_pos.y;
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

            // we need same point on same tile, but in square coords, to init movement
            let square_coord = iso_to_nav_coord(cursor_pos);
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

// The code after this comment is not related to `seldom_map_nav`

#[derive(Component)]
struct Player;

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
