//! Interoperability traits for positional components

use crate::prelude::*;

/// Component that represents a 2D position
pub trait Position2: Component {
    /// Get the position as a vector
    fn get(&self) -> Vec2;
    /// Set the position from a vector
    fn set(&mut self, pos: Vec2);
}

impl Position2 for Transform {
    fn get(&self) -> Vec2 {
        self.translation.truncate()
    }

    fn set(&mut self, pos: Vec2) {
        self.translation = pos.extend(self.translation.z);
    }
}
