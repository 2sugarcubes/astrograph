use coordinates::prelude::{Spherical, Vector3};
use quaternion::Quaternion;
use serde::{Deserialize, Serialize};

use crate::{consts::float, Float};

/// A struct that defines the rotation of a body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "SerializableRotating", into = "SerializableRotating")]
pub struct Rotating {
    /// The time for the body to rotate 360 degrees, as opposed to a [solar day](https://en.wikipedia.org/wiki/Synodic_day)
    sidereal_period: Float,
    /// The direction of the geographic north pole.
    axis: Vector3<Float>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SerializableRotating {
    sidereal_period: Float,
    axis: Spherical<Float>,
}

impl From<Rotating> for SerializableRotating {
    fn from(value: Rotating) -> Self {
        SerializableRotating {
            sidereal_period: value.sidereal_period,
            axis: value.axis.into(),
        }
    }
}

impl From<SerializableRotating> for Rotating {
    fn from(value: SerializableRotating) -> Self {
        Rotating {
            sidereal_period: value.sidereal_period,
            axis: value.axis.into(),
        }
    }
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

    /// Gets angle relative to the reference direction since last complete revolution
    fn get_mean_angle(&self, time: Float) -> Float {
        time % self.sidereal_period / self.sidereal_period * float::TAU
    }
}

#[cfg(test)]
mod test {
    use std::u8;

    use coordinates::{
        prelude::{Spherical, ThreeDimensionalConsts, Vector3},
        traits::{Magnitude, Positional},
    };
    use rand_distr::num_traits::float as _;

    use crate::{consts::float, Float};

    use super::Rotating;

    #[test]
    fn normalize_axis() {
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
    fn correct_quaternion_with_axis_up() {
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

    #[test]
    fn correct_quaternion_with_axis_right() {
        let rotations = Rotating::new(float::TAU, Spherical::RIGHT);
        let fixed_point = Vector3::FORWARD;

        for i in 0..u8::MAX {
            let angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;

            let (expected_z, expected_y) = (-angle).sin_cos();

            let [_, real_y, real_z] =
                quaternion::rotate_vector(rotations.get_rotation(angle), fixed_point.into());

            print!("Testing angle: {angle:.2}\t");

            assert_float_absolute_eq!(real_y, expected_y);
            assert_float_absolute_eq!(real_z, expected_z);

            println!("Passed ✅");
        }
    }

    #[test]
    fn correct_quaternion_with_axis_forward() {
        let rotations = Rotating::new(float::TAU, Spherical::FORWARD);
        let fixed_point = Vector3::UP;
        for i in 0..u8::MAX {
            let angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;

            let (expected_x, expected_z) = (-angle).sin_cos();

            let [real_x, _, real_z] =
                quaternion::rotate_vector(rotations.get_rotation(angle), fixed_point.into());

            print!("Testing angle: {angle:.2}\t");

            assert_float_absolute_eq!(real_x, expected_x);
            assert_float_absolute_eq!(real_z, expected_z);

            println!("Passed ✅");
        }
    }

    #[test]
    fn correct_quaternion_with_axis_not_on_great_circle() {
        let rotations = Rotating::new(float::TAU, Spherical::UP);

        for polar_angle in (0..u8::MAX).map(|x| Float::from(x) / Float::from(u8::MAX) * float::PI) {
            let fixed_point: Vector3<_> = Spherical::new(1.0, polar_angle, 0.0).into();
            for i in 0..u8::MAX {
                let angle = Float::from(i) / Float::from(u8::MAX) * float::TAU;

                let [real_x, real_y, real_z] =
                    quaternion::rotate_vector(rotations.get_rotation(angle), fixed_point.into());

                let (mut expected_y, mut expected_x) = (-angle).sin_cos();
                let expected_z = fixed_point.z;
                expected_x *= (1.0 - expected_z.powi(2)).sqrt();
                expected_y *= (1.0 - expected_z.powi(2)).sqrt();

                print!("Testing angle: {angle:.2}, at polar angle: {polar_angle:.2}");

                assert_float_absolute_eq!(real_z, expected_z);
                assert_float_absolute_eq!(real_y, expected_y);
                assert_float_absolute_eq!(real_x, expected_x);

                println!("Passed ✅");
            }
        }
    }
}
