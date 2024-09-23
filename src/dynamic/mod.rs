mod fixed;
mod keplerian;
mod simple;

use coordinates::three_dimensional::Vector3;

trait Dynamic {
    fn get_offset(&self, time: crate::FLOAT) -> Vector3<crate::FLOAT>;
}
