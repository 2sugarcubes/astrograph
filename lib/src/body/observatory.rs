use coordinates::prelude::{Spherical, ThreeDimensionalConsts, Vector3};
use log::warn;
use quaternion::Quaternion;
use serde::{Deserialize, Serialize};

use crate::{Float, LocalObservation};

use super::Arc;

/// Defines a place on the surface of a body where observations are made of the motion of bodies.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", from = "WeakObservatory")]
pub struct Observatory {
    /// A quaternion that encodes the rotation from the given longitude and latitude to the
    /// geographic north pole to make projections easier.
    location: Quaternion<Float>,
    /// The body that observations are being made from
    body: Arc,
    name: Result<String, Vec<usize>>,
}

impl Observatory {
    /// Generates an observatory on the given body and location.
    #[must_use]
    pub fn new(location: Spherical<Float>, body: Arc, name: Result<String, Vec<usize>>) -> Self {
        let location: Vector3<Float> = location.into();

        Self {
            location: quaternion::rotation_from_to(location.into(), Vector3::UP.into()),
            body,
            name,
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
    pub fn observe(&self, time: Float) -> Vec<LocalObservation> {
        if let Ok(body) = self.body.read() {
            let raw_observations = body.get_observations_from_here(time);

            // Rotate observations to put them in the local coordinate space from equitorial coordinate
            // space
            raw_observations
                .iter()
                .filter_map(|(body, pos)| {
                    let local_coordinates =
                        Vector3::from(quaternion::rotate_vector(self.location, (*pos).into()));
                    // FIXME: adjust z based on the body's radius since we aren't observing from the
                    // centre of the body

                    // Filter out bodies below the horizon
                    if local_coordinates.z >= 0.0 {
                        Some((body.clone(), local_coordinates.into()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            warn!("The body was poisoned, could not make observations from it");
            vec![]
        }
    }

    #[must_use]
    pub fn get_name(&self) -> String {
        let lat_long = Spherical::from(Vector3::from(quaternion::rotate_vector(
            self.location,
            Vector3::UP.into(),
        )));
        self.name.clone().unwrap_or_else(|id| {
            format!(
                "{}@{:.2}N{:.2}E",
                to_name(&id),
                lat_long.polar_angle.to_degrees() - 90.0,
                lat_long.azimuthal_angle.to_degrees() - 180.0
            )
        })
    }
}

#[derive(Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct WeakObservatory {
    location: Spherical<Float>,
    body_id: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// Converts a [`WeakObservatory`] to a regular [`Observatory`] by adding back reference counted
/// variables correctly.
///
/// # Panics
///
/// Panics if a body in the tree has a poisoned lock
pub fn to_observatory(weak_observatory: WeakObservatory, root: &Arc) -> Observatory {
    let mut body = root.clone();
    for child_id in &weak_observatory.body_id {
        // HACK: remove unwrap here, probably by returning an Option<Observatory>
        let b = body.read().unwrap().children[*child_id].clone();
        body = b;
    }

    Observatory::new(
        weak_observatory.location,
        body.clone(),
        weak_observatory.name.ok_or(weak_observatory.body_id),
    )
}

pub(super) fn to_name(id: &[usize]) -> String {
    if id.is_empty() {
        String::new()
    } else {
        let mut res = id[0].to_string();

        for i in &id[1..] {
            res.push_str(&format!("-{i}"));
        }

        res
    }
}

impl From<Observatory> for WeakObservatory {
    fn from(value: Observatory) -> Self {
        let (qw, [qx, qy, qz]) = value.location;
        // From https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToEuler/

        // asin(2*qx*qy + 2*qz*qw)
        // We take acos because we are using a polar angle, not a latitude
        let polar_angle = (2.0 - qx * qy + 2.0 * qz * qw).acos();

        // atan2(2*qy*qw-2*qx*qz , 1 - 2*qy^2 - 2*qz^2)
        let azimuthal_angle =
            (2.0 * qy * qw - 2.0 * qx * qz).atan2(1.0 - 2.0 * qy * qy - 2.0 * qz * qz);

        let body_id = value
            .body
            .read()
            .map(|body| body.get_id())
            .unwrap_or(vec![]);
        WeakObservatory {
            location: Spherical {
                polar_angle,
                azimuthal_angle,
                radius: 1.0,
            },
            body_id,
            name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use coordinates::prelude::{Spherical, ThreeDimensionalConsts, Vector3};

    use crate::{
        body::{observatory::WeakObservatory, rotating::Rotating, Arc, Body},
        dynamic::fixed::Fixed,
    };

    #[allow(dead_code)] // Will be useful if we rewrite that old test
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
    fn load_from_file() {
        let file = include_str!("../../../assets/solar-system.observatories.json");

        let observatories: Vec<WeakObservatory> = serde_json::from_str(file).unwrap();

        assert_eq!(observatories.len(), 6);
    }
}
