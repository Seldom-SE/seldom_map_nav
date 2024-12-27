use std::{collections::VecDeque, error::Error, time::Duration};

use mint::Vector3;
use navmesh::{NavPathMode, NavQuery};

use crate::{prelude::*, set::MapNavSet};

pub(crate) fn plug<P: Position2>(app: &mut App) {
    app.add_systems(
        Update,
        (apply_deferred, generate_paths::<P>, nav::<P>)
            .chain()
            .in_set(MapNavSet),
    );
}

/// A target to navigate to
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum PathTarget {
    /// A position
    Static(Vec2),
    /// An entity that has a position
    Dynamic(Entity),
}

/// Add this component to your entity to have it generate paths. Works as a state
/// in `seldom_state`.
#[derive(Clone, Component, Debug)]
pub struct Pathfind {
    /// Tilemap with the [`Navmeshes`] component
    pub map: Entity,
    /// Clearance radius
    pub radius: f32,
    /// How often to regenerate the path, if ever
    pub repath_frequency: Option<Duration>,
    /// Next time to repath
    pub next_repath: Duration,
    /// Target to navigate to
    pub target: PathTarget,
    /// Generated path
    pub path: VecDeque<Vec2>,
    /// Quality of querying a point on the navmesh
    pub query: NavQuery,
    /// Quality of finding a path
    pub path_mode: NavPathMode,
}

impl Pathfind {
    /// Create a `Pathfind`
    pub fn new(
        map: Entity,
        radius: f32,
        repath_frequency: Option<Duration>,
        target: PathTarget,
        query: NavQuery,
        path_mode: NavPathMode,
    ) -> Self {
        Self {
            map,
            radius,
            repath_frequency,
            next_repath: Duration::ZERO,
            target,
            path: default(),
            query,
            path_mode,
        }
    }
}

/// Add this component and [`Pathfind`] to your entity to have it navigate
#[derive(Clone, Component, Copy, Debug, Reflect)]
pub struct Nav {
    /// Speed by which to navigate
    pub speed: f32,
    /// Whether the entity has navigated to the destination
    pub done: bool,
}

impl Nav {
    /// Create a `Nav`
    pub fn new(speed: f32) -> Self {
        Self { speed, done: false }
    }
}

/// Components required for navigation
#[derive(Bundle, Clone, Debug)]
pub struct NavBundle {
    /// Pathfinding
    pub pathfind: Pathfind,
    /// Navigation
    pub nav: Nav,
}

pub(crate) fn generate_paths<P: Position2>(
    #[cfg(feature = "state")] mut commands: Commands,
    positions: Query<&P>,
    mut pathfinds: Query<(Entity, &P, &mut Pathfind)>,
    mut navs: Query<&mut Nav>,
    meshes: Query<&Navmeshes>,
    time: Res<Time>,
) {
    #[allow(unused_variables)]
    for (entity, position, mut pathfind) in &mut pathfinds {
        let repath = pathfind
            .repath_frequency
            .map(|repath_frequency| {
                let repath = pathfind.next_repath <= time.elapsed();
                if repath {
                    pathfind.next_repath = time.elapsed() + repath_frequency;
                }
                repath
            })
            .unwrap_or_else(|| {
                let path = pathfind.next_repath == Duration::ZERO;
                if path {
                    pathfind.next_repath = Duration::MAX;
                }
                path
            });

        if !repath {
            continue;
        }

        let path = || -> Result<VecDeque<Vec2>, Box<dyn Error>> {
            Ok(meshes
                .get(pathfind.map)?
                .mesh(pathfind.radius)
                .ok_or_else(|| {
                    format!(
                        "missing navmesh with clearance of at least {}",
                        pathfind.radius
                    )
                })?
                .find_path(
                    Vector3::from(position.get().extend(0.)).into(),
                    Vector3::from(
                        match pathfind.target {
                            PathTarget::Static(target) => target,
                            PathTarget::Dynamic(target) => positions.get(target)?.get(),
                        }
                        .extend(0.),
                    )
                    .into(),
                    pathfind.query,
                    pathfind.path_mode,
                )
                .ok_or("no valid path was found")?
                .into_iter()
                .map(|pos| Vec3::from(Vector3::from(pos)).truncate())
                .collect())
        }();

        #[cfg(feature = "log")]
        if let Err(error) = &path {
            warn!("failed to generate path: {error}");
        }
        #[cfg(feature = "state")]
        let failure = path.is_err();
        pathfind.path = path.unwrap_or_default();

        let Ok(mut nav) = navs.get_mut(entity) else {
            continue;
        };

        nav.done = pathfind.path.is_empty();

        #[cfg(feature = "state")]
        if failure {
            commands.entity(entity).insert(Done::Failure);
        }
    }
}

fn nav<P: Position2>(
    #[cfg(feature = "state")] mut commands: Commands,
    mut navs: Query<(Entity, &mut P, &mut Pathfind, &mut Nav)>,
    time: Res<Time>,
) {
    #[allow(unused_variables)]
    for (entity, mut position, mut pathfind, mut nav) in &mut navs {
        if pathfind.path.is_empty() {
            #[cfg(feature = "state")]
            commands.entity(entity).insert(Done::Success);
            continue;
        }

        let mut pos = position.get();
        let mut travel_dist = nav.speed * time.delta_secs();
        let mut dest;
        let mut dest_dist;

        while travel_dist >= {
            dest = *pathfind.path.front().unwrap();
            dest_dist = (dest - pos).length();
            dest_dist
        } {
            pos = dest;
            travel_dist -= dest_dist;

            pathfind.path.pop_front();
            if pathfind.path.is_empty() {
                break;
            }
        }

        if pathfind.path.is_empty() {
            nav.done = true;
            #[cfg(feature = "state")]
            commands.entity(entity).insert(Done::Success);
        } else {
            let delta = (dest - pos).normalize() * travel_dist;
            pos += delta;
        }

        position.set(pos);
    }
}
