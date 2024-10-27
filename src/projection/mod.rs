use coordinates::prelude::{Spherical, Vector2};

use crate::{consts::float, Float};

/// Trait that encapsulates the core functionality of a projector, a set of equations that convert
/// a point from three-dimensional space onto a two-dimensional plane.
pub trait Projection {
    /// Projects from 3D to 2D while taking into account any state the projector has.
    //#[inline(always)]
    fn project_with_state(&self, location: &Spherical<Float>) -> Option<Vector2<Float>> {
        // default behaviour when there is no state
        Self::project(location)
    }
    /// Projects from 3D to 2D without taking into account any state the projector has, generally
    /// centered on the Z axis (up/down), but check the implementation you are using to be sure.
    fn project(location: &Spherical<Float>) -> Option<Vector2<Float>>;
}

/// An [orthographic projector](https://en.wikipedia.org/wiki/Orthographic_map_projection) that is centered on the positive z direction, but thanks to the output
/// of [crate::body::observatory::Observatory::observe] observations are already centered on the z axis.
#[derive(Debug, Clone, Copy)]
pub struct StatelessOrthographic();

impl Projection for StatelessOrthographic {
    /// # Returns
    ///
    /// None if the point cannot be projected i.e. it is over the horizon, the projected point
    /// otherwise.
    fn project(location: &Spherical<Float>) -> Option<Vector2<Float>> {
        // If the location is on the other hemisphere
        if location.polar_angle > float::FRAC_PI_2 {
            return None;
        }
        // cos/sin swapped because 90deg north is our zero point, not the equator
        let lat_sin = location.polar_angle.sin();
        let (azi_sin, azi_cos) = location.azimuthal_angle.sin_cos();

        // Since the zero point is 0,0 we can ignore sin(lat_0) * cos(lat)
        Some(Vector2 {
            x: lat_sin * -azi_sin,
            y: lat_sin * -azi_cos,
        })
    }
}

/// An [orthographic projector](https://en.wikipedia.org/wiki/Orthographic_map_projection) that is centered on an orbitrary longitude and latitude. In most cases it will be quicker to use the [self::StatelessOrthographic] projection.
#[derive(Debug, Clone)]
pub struct Orthographic(Float, Float);

impl Projection for Orthographic {
    fn project_with_state(&self, location: &Spherical<Float>) -> Option<Vector2<Float>> {
        let (long_sin, long_cos) = (location.azimuthal_angle - self.1).sin_cos();

        // cos/sin swapped because 90deg north is our zero point, not the equator
        let (lat_cos, lat_sin) = location.polar_angle.sin_cos();
        let (lat_zero_cos, lat_zero_sin) = self.0.sin_cos();

        if lat_zero_sin * lat_sin + lat_zero_cos * lat_cos * long_cos < 0.0 - Float::EPSILON {
            // Clip it out because it is on the other hemisphere
            None
        } else {
            Some(Vector2 {
                x: lat_cos * -long_sin,
                y: lat_zero_cos * lat_sin - lat_zero_sin * lat_cos * long_cos,
            })
        }
    }

    //#[inline(always)]
    fn project(location: &Spherical<Float>) -> Option<Vector2<Float>> {
        StatelessOrthographic::project(location)
    }
}

// TODO make macro for this (will speed up inplementing projections)
#[cfg(test)]
mod test {
    mod stateless {
        use coordinates::prelude::ThreeDimensionalConsts;

        use super::super::*;
        #[test]
        fn up_maps_to_0_0() {
            let input = Spherical::UP;
            let output = StatelessOrthographic::project(&input).unwrap();

            println!("Expected: (0.0, 0.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn down_maps_to_none() {
            let input = Spherical::DOWN;
            let output = StatelessOrthographic::project(&input);

            assert_eq!(output, None);
        }

        #[test]
        fn north_maps_to_0_1() {
            let input = Spherical::LEFT;
            println!("{input:?}");
            let output = StatelessOrthographic::project(&input).unwrap();

            println!("Expected: (0.0, 1.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, 1.0);
        }

        #[test]
        fn west_maps_to_neg_1_0() {
            let input = Spherical::FORWARD;
            let output = StatelessOrthographic::project(&input).unwrap();

            println!("Expected: (-1.0, 0.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, -1.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn east_maps_to_1_0() {
            let input = Spherical::BACK;
            let output = StatelessOrthographic::project(&input).unwrap();

            println!("Expected: (1.0, 0.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 1.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn south_maps_to_0_neg_1() {
            let input = Spherical::RIGHT;
            let output = StatelessOrthographic::project(&input).unwrap();

            println!("Expected: (0.0, -1.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, -1.0);
        }
    }

    mod stateful {
        use coordinates::prelude::ThreeDimensionalConsts;

        use crate::consts::float;

        use super::super::*;

        // [Null Island](https://en.wikipedia.org/wiki/Null_Island) equivalent
        const PROJECTOR: Orthographic = Orthographic(float::FRAC_PI_2, 0.0);

        #[test]
        fn up_maps_to_0_0() {
            let input = Spherical::RIGHT;
            println!("{input:?}");
            let output = PROJECTOR.project_with_state(&input).unwrap();

            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn north_maps_to_0_1() {
            let input = Spherical::UP;
            let output = PROJECTOR.project_with_state(&input).unwrap();

            println!("Expected: (0.0, 1.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, 1.0);
        }

        #[test]
        fn west_maps_to_neg_1_0() {
            let input = Spherical::FORWARD;
            let output = PROJECTOR.project_with_state(&input).unwrap();

            assert_float_absolute_eq!(output.x, -1.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn east_maps_to_1_0() {
            let input = Spherical::BACK;
            let output = PROJECTOR.project_with_state(&input).unwrap();

            assert_float_absolute_eq!(output.x, 1.0);
            assert_float_absolute_eq!(output.y, 0.0);
        }

        #[test]
        fn south_maps_to_0_neg_1() {
            let input = Spherical::DOWN;
            let output = PROJECTOR.project_with_state(&input).unwrap();

            println!("Expected: (0.0, -1.0)\t Real: {output:.2}");
            assert_float_absolute_eq!(output.x, 0.0);
            assert_float_absolute_eq!(output.y, -1.0);
        }
    }
}
