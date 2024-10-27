use coordinates::three_dimensional::Vector3;

use crate::Float;

use super::Dynamic;

/// Returns a struct that returns the same offset for any given time. Especially useful for distant
/// objects or very slow moving objects
#[derive(Clone, Copy, Debug)]
pub struct Fixed(pub(crate) Vector3<Float>);
impl Fixed {
    /// Returns a new fixed dynamic at the given location
    #[must_use]
    pub fn new(location: Vector3<Float>) -> Self {
        Fixed(location)
    }
}

impl Dynamic for Fixed {
    fn get_offset(&self, _: crate::Float) -> Vector3<crate::Float> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use coordinates::prelude::*;

    use super::*;
    #[test]
    fn location_from_time() {
        let fixed_up = Fixed(Vector3::UP);
        let fixed_right = Fixed(Vector3::RIGHT);
        let fixed_back = Fixed(Vector3::BACK);

        for t in 0_u8..10 {
            assert_eq!(fixed_up.get_offset(Float::from(t)), Vector3::UP);
            assert_eq!(fixed_right.get_offset(Float::from(t)), Vector3::RIGHT);
            assert_eq!(fixed_back.get_offset(Float::from(t)), Vector3::BACK);
        }
    }
}
