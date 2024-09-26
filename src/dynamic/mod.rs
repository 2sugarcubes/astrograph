pub mod fixed;
pub mod keplerian;
//mod simple;

use std::fmt::Debug;

use coordinates::three_dimensional::Vector3;
use dyn_clone::DynClone;

use crate::Float;

pub trait Dynamic: Debug + DynClone {
    #[must_use]
    fn get_offset(&self, time: Float) -> Vector3<Float>;
}

dyn_clone::clone_trait_object!(Dynamic);
