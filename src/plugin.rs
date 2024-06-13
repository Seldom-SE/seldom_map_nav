use std::marker::PhantomData;

use crate::{nav::plug, prelude::*};

/// Add to your app to enable pathing and navigation. The type parameter accepts
/// the position component used by your navigators.
#[derive(Debug)]
pub struct MapNavPlugin<P: Position2 = Transform>(PhantomData<P>);

impl<P: Position2> Plugin for MapNavPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_plugins(plug::<P>);
    }
}

impl<P: Position2> Default for MapNavPlugin<P> {
    fn default() -> Self {
        Self(default())
    }
}
