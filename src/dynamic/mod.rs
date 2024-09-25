pub mod fixed;
pub mod keplerian;
//mod simple;

use coordinates::three_dimensional::Vector3;

pub trait Dynamic {
    #[must_use]
    fn get_offset(&self, time: crate::Float) -> Vector3<crate::Float>;
}
