use coordinates::prelude::{Spherical, ThreeDimensionalConsts, Vector3};
use quaternion::Quaternion;

use crate::Float;

use super::ArcBody;

pub struct Observatory {
    location: Quaternion<Float>,
    body: ArcBody,
}

impl Observatory {
    pub fn new(location: Spherical<Float>, body: ArcBody) -> Self {
        let location: Vector3<Float> = location.into();
        Self {
            location: quaternion::rotation_from_to(location.into(), Vector3::UP.into()),
            body,
        }
    }

    /// Takes bodies from a universal coordinate space and converts them to local coordinates
    /// relative to the observatory
    pub fn observe(&self, time: Float) -> Vec<(ArcBody, Vector3<Float>)> {
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
            .map(|(body, pos)| {
                (
                    body.clone(),
                    quaternion::rotate_vector(rotation, (*pos).into()).into(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    //FIXME write some tests later, my head hurts
}
