use coordinates::prelude::{Spherical, ThreeDimensionalConsts, Vector3};
use quaternion::Quaternion;

use crate::Float;

use super::Arc;

/// Defines a place on the surface of a body where observations are made of the motion of bodies.
#[derive(Debug, Clone)]
pub struct Observatory {
    /// A quaternion that encodes the rotation from the given longitude and latitude to the
    /// geographic north pole to make projections easier.
    location: Quaternion<Float>,
    /// The body that observations are being made from
    body: Arc,
}

impl Observatory {
    /// Generates an observatory on the given body and location.
    #[must_use]
    pub fn new(location: Spherical<Float>, body: Arc) -> Self {
        let location: Vector3<Float> = location.into();
        Self {
            location: quaternion::rotation_from_to(location.into(), Vector3::UP.into()),
            body,
        }
    }

    /// Takes bodies from a universal coordinate space and converts them to local coordinates
    /// relative to the observatory
    ///
    /// # Panics
    ///
    /// If it cannot get a clean read lock on the body this observatory is on. i.e. the [`std::sync::RwLock`] is
    /// [poisoned](https://doc.rust-lang.org/std/sync/struct.RwLock.html#poisoning).
    #[must_use]
    pub fn observe(&self, time: Float) -> Vec<(Arc, Spherical<Float>)> {
        let body = self.body.read().unwrap();
        let raw_observations = body.get_observations_from_here(time);

        let rotation = if let Some(rotation) = &body.rotation {
            quaternion::mul(self.location, rotation.get_rotation(time))
        } else {
            self.location
        };

        // Rotate observations to put them in the local coordinate space
        raw_observations
            .iter()
            .filter_map(|(body, pos)| {
                let local_coordinates =
                    Vector3::from(quaternion::rotate_vector(rotation, (*pos).into()));

                // Filter out bodies below the horizon
                if local_coordinates.z >= 0.0 {
                    Some((body.clone(), local_coordinates.into()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    //FIXME write some tests later, my head hurts

    use coordinates::prelude::{Spherical, ThreeDimensionalConsts, Vector3};

    use crate::{
        body::{observatory::Observatory, rotating::Rotating, Arc, Body},
        consts::float,
        dynamic::fixed::Fixed,
        Float,
    };

    fn get_toy_example_body() -> Arc {
        let body = Body::new(None, Fixed::new(Vector3::ORIGIN));
        body.write().unwrap().rotation = Some(Rotating::new(4.0, Spherical::UP));
        let _ = Body::new(Some(body.clone()), Fixed::new(Vector3::RIGHT));
        //                        \-> ONCE TOLD ME. Now you can't get it out of your head either
        //let _ = Body::new(Some(body.clone()), Fixed::new(Vector3::BACK));
        //let _ = Body::new(Some(body.clone()), Fixed::new(Vector3::LEFT));
        //let _ = Body::new(Some(body.clone()), Fixed::new(Vector3::FORWARD));

        body
    }

    #[test]
    fn simple_rotation_test() {
        let root = get_toy_example_body();
        let observatory = Observatory::new(Spherical::RIGHT, root);

        for (time, polar_angle) in [
            (0_u8, 0.0),
            (1, float::FRAC_PI_2),
            (2, float::PI),
            (3, float::FRAC_PI_2),
        ] {
            let observations: Vec<Spherical<Float>> = observatory
                .observe(Float::from(time))
                .iter()
                .map(|(_, loc)| *loc)
                .collect();

            println!("{observations:.2?}");

            if observations.is_empty() {
                // Increased precision leads to there correctly not being any bodies above the
                // horizon at t=2 or t=3 when f64 is used.
                assert!(time == 2 || time == 3 && cfg!(feature = "f64"));
            } else {
                assert_float_absolute_eq!(observations[0].polar_angle, polar_angle);
            }
        }
    }
}
