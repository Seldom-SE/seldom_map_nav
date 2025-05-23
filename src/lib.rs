//! Bevy plugin that does navmesh generation, pathfinding, and navigation for tilemaps.
//! Navmesh generation is available without Bevy dependency.

#![warn(missing_docs)]

pub mod mesh;
#[cfg(feature = "bevy")]
mod nav;
#[cfg(feature = "bevy")]
mod plugin;
#[cfg(feature = "bevy")]
mod pos;
#[cfg(feature = "bevy")]
pub mod set;
mod vertex;

/// Module for convenient imports. Use with `use seldom_map_nav::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "log")]
    pub(crate) use bevy_log::prelude::*;
    #[cfg(feature = "state")]
    pub(crate) use seldom_state::prelude::*;
    #[cfg(feature = "bevy")]
    pub(crate) use {
        bevy_app::prelude::*, bevy_ecs::prelude::*, bevy_math::prelude::*,
        bevy_reflect::prelude::*, bevy_time::prelude::*, bevy_transform::prelude::*,
    };

    pub use crate::mesh::{Navability, Navmeshes};
    #[cfg(feature = "bevy")]
    pub use crate::{
        nav::{Nav, NavBundle, PathTarget, Pathfind},
        plugin::MapNavPlugin,
        pos::Position2,
    };
    pub use navmesh::{NavPathMode, NavQuery};
}
