use coordinates::prelude::{Spherical, Vector2};

use crate::Float;

pub mod orthographic;

pub trait Projection {
    #[inline(always)]
    fn project_with_state(&self, location: &Spherical<Float>) -> Option<Vector2<Float>> {
        // default behaviour when there is no state
        Self::project(location)
    }
    fn project(location: &Spherical<Float>) -> Option<Vector2<Float>>;
}
