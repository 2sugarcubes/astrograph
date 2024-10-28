use coordinates::prelude::{Spherical, Vector3};
use quaternion::Quaternion;

use crate::{consts::float, Float};

/// A struct that defines the rotation of a body.
#[derive(Debug, Clone)]
pub struct Rotating {
    /// The time for the body to rotate 360 degrees, as opposed to a [solar day](https://en.wikipedia.org/wiki/Synodic_day)
    sidereal_period: Float,
    /// The direction of the geographic north pole.
    axis: Vector3<Float>,
}

impl Rotating {
    /// Creates a rotating body type that rotates relative to an infinitely distant point every
    /// `sidereal_period` hours, and rotates around the given `axis` in a counter-clockwise manner
    /// (like the earth's [geographic north pole](https://en.wikipedia.org/wiki/North_Pole))
    #[must_use]
    pub fn new(sidereal_period: Float, mut axis: Spherical<Float>) -> Self {
        // Set axis to a unit vector
        axis.radius = 1.0;
        Self {
            sidereal_period,
            axis: axis.into(),
        }
    }

    /// Returns a rotation for a given time.
    #[must_use]
    pub fn get_rotation(&self, time: Float) -> Quaternion<Float> {
        quaternion::axis_angle(self.axis.into(), -self.get_mean_angle(time))
    }

    /// Gets angle relative to the referance direction since last complete revolution
    fn get_mean_angle(&self, time: Float) -> Float {
        time % self.sidereal_period / self.sidereal_period * float::TAU
    }
}

#[cfg(test)]
mod test {
    use coordinates::{
        prelude::{Spherical, ThreeDimensionalConsts, Vector3},
        traits::Magnitude,
    };

    use crate::{consts::float, Float};

    use super::Rotating;

    #[test]
    fn normalise_axis() {
        const EXPECTED_MAGNITUDE: Float = 1.0;
        let axis_small = Spherical {
            radius: 0.5,
            polar_angle: 0.01,
            azimuthal_angle: 0.01,
        };
        let axis_large = Spherical {
            radius: 10.0,
            polar_angle: 0.01,
            azimuthal_angle: 0.01,
        };

        let small_rotating_body = Rotating::new(1.0, axis_small);
        assert_float_absolute_eq!(small_rotating_body.axis.magnitude(), EXPECTED_MAGNITUDE);

        let large_rotating_body = Rotating::new(1.0, axis_large);
        assert_float_absolute_eq!(large_rotating_body.axis.magnitude(), EXPECTED_MAGNITUDE);
    }

    #[test]
    fn correct_rotations() {
        // Rotate around the y axis with a period of tau so that time should equal the expected
        // angle
        let rotations = Rotating::new(float::TAU, Spherical::UP);

        for i in 0_u8..u8::MAX {
            let expected_angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;

            assert_float_absolute_eq!(rotations.get_mean_angle(expected_angle), expected_angle);
        }
    }

    #[test]
    fn correct_quaternion() {
        let rotations = Rotating::new(float::TAU, Spherical::UP);
        let fixed_point = Vector3::RIGHT;

        for i in 0..u8::MAX {
            let angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;
            // Negative because the apparent rotation of the fixed body will be opposite relative
            // to our motion
            let (expected_y, expected_x) = (-angle).sin_cos();

            // Rotate the fixed point by the amount our rotating body has rotated
            let [real_x, real_y, _] =
                quaternion::rotate_vector(rotations.get_rotation(angle), fixed_point.into());

            print!("Testing angle: {angle:.2}\t");

            assert_float_absolute_eq!(real_x, expected_x);
            assert_float_absolute_eq!(real_y, expected_y);

            println!("Passed ✅");
        }
    }
}
