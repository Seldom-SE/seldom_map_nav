// Here is an extended nav example, where we switch Player between Idle and GoTo states

use bevy::{prelude::*, sprite::Anchor};
use rand::{thread_rng, Rng};
use seldom_map_nav::prelude::*;
use seldom_map_nav::set::MapNavSet;
use seldom_state::prelude::*;

#[derive(Clone, Reflect)]
struct Click;

// Triggers are used to create transitions between states
impl OptionTrigger for Click {
    // Immutable system parameters are accessed here
    type Param<'w, 's> = (Res<'w, Input<MouseButton>>, Res<'w, CursorPos>);
    type Some = Vec2;

    fn trigger(&self, _: Entity, (mouse, cursor_position): Self::Param<'_, '_>) -> Option<Vec2> {
        mouse
            .just_pressed(MouseButton::Left)
            .then_some(())
            // Return any needed trigger data here. Here we return a `Vec2` that the player will
            // move to
            .and(**cursor_position)
    }
}

// Two states that we will switch between, using SparseSet instead of Table for fast insertion and
// removal

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
        .add_plugins((
            DefaultPlugins,
            StateMachinePlugin,
            MapNavPlugin::<Transform>::default(),
        ))
        .init_resource::<CursorPos>()
        .add_systems(Startup, init)
        .add_systems(
            Update,
            (
                update_cursor_pos,
                // This system ordering prevents the player from getting stuck idle
                move_player.before(MapNavSet),
            )
                .chain(),
        )
        .run();
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_bundle = init_inner(&mut commands, &asset_server);

    commands.spawn((
        player_bundle,
        StateMachine::default()
            // When the player clicks, go there. Using previously defined `Click` trigger here
            .trans_builder(Click, |_: &AnyState, pos| {
                Some(GoToSelection {
                    speed: 200.,
                    target: pos,
                })
            })
            // `DoneTrigger` triggers when the `Done` component is added to the entity. When they're
            // done going to the selection, idle.
            .trans::<GoToSelection>(DoneTrigger::Success, Idle)
            .set_trans_logging(true),
        Player,
        Idle,
    ));
}

// Navigate the player to wherever you click
fn move_player(
    mut commands: Commands,
    players: Query<(Entity, &GoToSelection), Added<GoToSelection>>,
    navmeshes: Query<Entity, With<Navmeshes>>,
) {
    // `GoToSelection` component was added by the state machine after `Click` triggers
    for (entity, go_to_selection) in &players {
        commands.entity(entity).insert(NavBundle {
            pathfind: Pathfind::new(
                navmeshes.single(),
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

// The code after this comment is less relevant to this particular example

const MAP_SIZE: UVec2 = UVec2::new(24, 24);
const TILE_SIZE: Vec2 = Vec2::new(32., 32.);
// This is the radius of a square around the player that should not intersect with the terrain
const PLAYER_CLEARANCE: f32 = 8.;

fn init_inner(commands: &mut Commands, asset_server: &Res<AssetServer>) -> SpriteBundle {
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

    // Spawn the tilemap with a `Navmeshes` component
    commands
        .spawn(Navmeshes::generate(MAP_SIZE, TILE_SIZE, navability, [PLAYER_CLEARANCE]).unwrap());

    SpriteBundle {
        transform: Transform::from_translation((player_pos + TILE_SIZE / 2.).extend(1.)),
        texture: asset_server.load("player.png"),
        ..default()
    }
}

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
