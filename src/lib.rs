//! Bevy plugin that does navmesh generation, pathfinding, and navigation for tilemaps.
//! Navmesh generation is available without Bevy dependency.

#![warn(missing_docs)]

pub mod mesh;
#[cfg(feature = "bevy")]
mod nav;
#[cfg(feature = "bevy")]
mod plugin;
#[cfg(feature = "bevy")]
pub mod set;
mod vertex;

/// Module for convenient imports. Use with `use seldom_map_nav::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "bevy")]
    pub(crate) use bevy::prelude::*;
    #[cfg(feature = "bevy")]
    pub(crate) use seldom_interop::prelude::*;
    #[cfg(feature = "state")]
    pub(crate) use seldom_state::prelude::*;

    pub use crate::mesh::{Navability, Navmeshes};
    #[cfg(feature = "bevy")]
    pub use crate::{
        nav::{Nav, NavBundle, PathTarget, Pathfind},
        plugin::{map_nav_plugin, MapNavPlugin},
    };
    pub use navmesh::{NavPathMode, NavQuery};
}
