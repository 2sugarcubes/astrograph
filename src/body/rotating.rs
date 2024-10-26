use coordinates::prelude::{Spherical, Vector3};
use quaternion::Quaternion;

use crate::{consts::float, Float};

#[derive(Debug, Clone)]
pub struct Rotating {
    sidereal_period: Float,
    axis: Vector3<Float>,
}

impl Rotating {
    pub fn new(sidereal_period: Float, axis: Spherical<Float>) -> Self {
        // Set axis to a unit vector
        let axis = if axis.radius == 1.0 {
            axis
        } else {
            let mut axis = axis.clone();
            axis.radius = 1.0;
            axis
        };
        Self {
            sidereal_period,
            axis: axis.into(),
        }
    }

    pub fn get_rotation(&self, time: Float) -> Quaternion<Float> {
        quaternion::axis_angle(self.axis.into(), -self.get_mean_angle(time))
    }

    fn get_mean_angle(&self, time: Float) -> Float {
        time % self.sidereal_period / self.sidereal_period * float::TAU
    }
}

#[cfg(test)]
mod test {
    use std::u8;

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
        assert_eq!(small_rotating_body.axis.magnitude(), EXPECTED_MAGNITUDE);

        let large_rotating_body = Rotating::new(1.0, axis_large);
        assert_eq!(large_rotating_body.axis.magnitude(), EXPECTED_MAGNITUDE);
    }

    #[test]
    fn correct_rotations() {
        // Rotate around the y axis with a period of tau so that time should equal the expected
        // angle
        let rotations = Rotating::new(float::TAU, Spherical::UP);

        for i in 0_u8..u8::MAX {
            let expected_angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;
            let observed_angle = rotations.get_mean_angle(expected_angle);

            assert!(
                (rotations.get_mean_angle(expected_angle) - expected_angle).abs() < Float::EPSILON,
                "{observed_angle:.2} is too far from the expected angle {expected_angle:.2}"
            );
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
