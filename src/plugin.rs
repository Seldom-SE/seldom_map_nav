use std::marker::PhantomData;

use crate::{nav::nav_plugin, prelude::*, set::MapNevSet};
use seldom_fn_plugin::FnPluginExt;

/// Add to your app to enable pathing and navigation. The type parameter accepts
/// the position component used by your navigators.
#[derive(Debug)]
pub struct MapNavPlugin<P: Position2<Position = Vec2> = Transform>(PhantomData<P>);

impl<P: Position2<Position = Vec2>> Plugin for MapNavPlugin<P> {
    fn build(&self, app: &mut App) {
        app.fn_plugin(map_nav_plugin::<P>);
    }
}

impl<P: Position2<Position = Vec2>> Default for MapNavPlugin<P> {
    fn default() -> Self {
        Self(default())
    }
}

/// Function called by [`MapNavPlugin`]. You may instead call it directly
/// or use `seldom_fn_plugin`, which is another crate I maintain.
pub fn map_nav_plugin<P: Position2<Position = Vec2>>(app: &mut App) {
    app.configure_set(MapNevSet.in_base_set(CoreSet::Update))
        .fn_plugin(nav_plugin::<P>);
}
