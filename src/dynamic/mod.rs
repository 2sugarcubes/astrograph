/// A dynamic that stays in a constant location
pub mod fixed;
/// A dynamic that fits the [keplerian laws for planetary motion](https://en.wikipedia.org/wiki/Kepler%27s_laws_of_planetary_motion).
pub mod keplerian;
//mod circular;

use std::fmt::Debug;

use coordinates::three_dimensional::Vector3;
use dyn_clone::DynClone;
use dyn_partial_eq::dyn_partial_eq;

use crate::Float;

/// Interface to be used by any dynamic.
#[typetag::serde]
#[dyn_partial_eq]
pub trait Dynamic: Debug + Send + Sync + DynClone {
    /// Returns the position relative to the parent body at a given time.
    #[must_use]
    fn get_offset(&self, time: Float) -> Vector3<Float>;
}

dyn_clone::clone_trait_object!(Dynamic);
