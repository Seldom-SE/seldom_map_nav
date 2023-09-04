// Here is an extended nav example, where we switch Player between Idle and GoTo states

use bevy::{prelude::*, sprite::Anchor};
use rand::{thread_rng, Rng};
use seldom_map_nav::prelude::*;
use seldom_state::prelude::*;
use seldom_map_nav::set::MapNavSet;

#[derive(Clone, Reflect)]
struct Click;

// implementing `OptionTrigger` trait will allow to use Click in transition building later
impl OptionTrigger for Click {
    // any immutable resources allowed here
    type Param<'w, 's> = (Res<'w, Input<MouseButton>>, Res<'w, CursorPos>);
    type Some = Vec2;

    fn trigger(&self, _: Entity, (mouse, cursor_position): Self::Param<'_, '_>) -> Option<Vec2> {
        mouse
            .just_pressed(MouseButton::Left)
            .then_some(())
            // return any necessary trigger data here. here we return Vec2 that Player will move to
            .and(**cursor_position)
    }
}

// define 2 states we will switch between, with triggers
// using SparseSet for quick adding/removing, instead of Table
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct GoToSelection {
    speed: f32,
    target: Vec2,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StateMachinePlugin, MapNavPlugin::<Transform>::default()))
        .init_resource::<CursorPos>()
        .add_systems(Startup, init)
        .add_systems(Update, (
            update_cursor_pos,
            // It's important to start movement before pathfind logic, or we will get Idle message just after we click
            move_player.before(MapNavSet)
        ).chain())
        .run();
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    //moved `nav` related code to inner function
    let player_bundle = _init(&mut commands, &asset_server);

    commands.spawn((
        player_bundle,
        StateMachine::default()
            // When the player clicks, go there. Using previously defined `Click` trigger gere
            .trans_builder(Click, |_: &AnyState, pos| {
                Some(GoToSelection {
                    speed: 200.,
                    target: pos,
                })
            })
            // `DoneTrigger` triggers when the `Done` component is added to the entity. When they're
            // done going to the selection, idle.
            .trans::<GoToSelection>(DoneTrigger::Success, Idle)
            // here you can modify any entity components you need, e.x. animation state
            .on_enter::<Idle>(|_entity_commands| {
                // _entity_commands.remove(some_component)
                info!("IDLE!");
            })
            .on_enter::<GoToSelection>(|_entity_commands| {
                // _entity_commands.insert(some_component)
                info!("GOTO!");
            })
            .set_trans_logging(true),
        Player,
        Idle
    ));
}

// Navigate the player to wherever you click
fn move_player(
    mut commands: Commands,
    players: Query<(Entity, &GoToSelection), Added<GoToSelection>>,
    navmesheses: Query<Entity, With<Navmeshes>>,
) {
    // GoToSelection component was added by state machine system after Click trigger
    for (entity, go_to_selection) in &players {
        commands.entity(entity).insert(NavBundle {
            pathfind: Pathfind::new(
                navmesheses.single(),
                PLAYER_CLEARANCE,
                None,
                PathTarget::Static(go_to_selection.target),
                NavQuery::Accuracy,
                NavPathMode::Accuracy,
            ),
            nav: Nav::new(go_to_selection.speed),
        });
    }
}

// The code after relates to `nav` example
const MAP_SIZE: UVec2 = UVec2::new(24, 24);
const TILE_SIZE: Vec2 = Vec2::new(32., 32.);
// This is the radius of a square around the player that should not intersect with the terrain
const PLAYER_CLEARANCE: f32 = 8.;

fn _init(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>
) -> SpriteBundle {
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

    SpriteBundle {
        transform: Transform::from_translation((player_pos + TILE_SIZE / 2.).extend(1.)),
        texture: asset_server.load("player.png"),
        ..default()
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
