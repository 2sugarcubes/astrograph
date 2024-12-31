use crate::{consts::GRAVITATIONAL_CONSTANT, Float};
use coordinates::prelude::*;
use dyn_partial_eq::DynPartialEq;
use quaternion::Quaternion;
use serde::{Deserialize, Serialize};

use super::Dynamic;

/// Struct that best fits [kepler's laws of planetary
/// motion](https://en.wikipedia.org/wiki/Kepler%27s_laws_of_planetary_motion).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, DynPartialEq)]
#[serde(from = "IntermediateKeplerian", into = "IntermediateKeplerian")]
pub struct Keplerian {
    // Size and shape
    /// Unit: unitless.
    ///
    /// Definition: How circular the orbit is.
    pub(crate) eccentricity: Float,
    /// Unit: light-seconds.
    ///
    /// Definition: Half the length of the longest diameter through the ellipsis.
    pub(crate) semi_major_axis: Float,

    // Orbital Plane, and argument of ascending node, argument of periapsis, and inclination.
    /// Unit: radians, sort of.
    ///
    /// Definition: This variable encodes how the orbit is rotated relative to a reference
    /// direction. encompassing the argument of the periapsis, the orbital inclination, and the
    /// argument of the ascending node.
    inclination: Quaternion<Float>,

    /// Unit: radian
    ///
    /// Definition: How far along the orbit this body was at the "start of time" (t=0)
    mean_anomaly_at_epoch: Float,

    /// Unit: Hours
    ///
    /// Definition: How long it takes for this body to complete one orbit (when the angle between an
    /// infinitely distant point and the parent body are equal again i.e. the [sidereal period](https://en.wikipedia.org/wiki/Orbital_period#Related_periods) as opposed to [tropical period](https://en.wikipedia.org/wiki/Solar_year), or [synodic period](https://en.wikipedia.org/wiki/Orbital_period#Synodic_period))
    orbital_period: Float,

    calculated_fields: CalculatedFields,
}

impl PartialEq for Keplerian {
    fn eq(&self, other: &Self) -> bool {
        self.eccentricity == other.eccentricity
            && self.semi_major_axis == other.semi_major_axis
            && self.mean_anomaly_at_epoch == other.mean_anomaly_at_epoch
            && self.orbital_period == other.orbital_period
            && self.inclination == other.inclination
    }
}

#[derive(Debug, Clone, Copy)]
struct CalculatedFields {
    longitude_of_ascending_node: Float,
    argument_of_periapsis: Float,
    inclination: Float,
}

impl Keplerian {
    /// Generates a new keplerian dynamic with the calculated fields populated
    #[must_use]
    pub fn new(
        eccentricity: Float,
        semi_major_axis: Float,
        inclination: Float,
        longitude_of_ascending_node: Float,
        argument_of_periapsis: Float,
        true_anomaly: Float,
        parent_mass: Float,
    ) -> Self {
        let orbital_period = Float::TAU
            * (semi_major_axis * semi_major_axis * semi_major_axis
                / (parent_mass * GRAVITATIONAL_CONSTANT))
                .sqrt();
        Self::new_with_period(
            eccentricity,
            semi_major_axis,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
            orbital_period,
        )
    }

    /// Generates a new Keplerian dynamic with the calculated fields populated, assuming you know
    /// the period of this orbit before hand.
    #[must_use]
    pub fn new_with_period(
        eccentricity: Float,
        semi_major_axis: Float,
        inclination: Float,
        longitude_of_ascending_node: Float,
        argument_of_periapsis: Float,
        mean_anomaly_at_epoch: Float,
        orbital_period: Float,
    ) -> Self {
        let inclination_quaternion =
            quaternion::euler_angles(0.0, longitude_of_ascending_node, inclination);
        let inclination_quaternion = quaternion::mul(
            inclination_quaternion,
            quaternion::axis_angle(
                [0.0, 1.0, 0.0],
                argument_of_periapsis + longitude_of_ascending_node,
            ),
        );

        Self {
            eccentricity,
            semi_major_axis,
            inclination: inclination_quaternion,
            mean_anomaly_at_epoch,
            orbital_period,
            calculated_fields: CalculatedFields { longitude_of_ascending_node, argument_of_periapsis, inclination },
        }
    }

    /// Calculates the mean anomaly from the time since the epoch
    /// Note: May be larger than Tau, but should be fine since it will be used in sin or cos
    /// functions
    fn get_mean_anomaly(&self, time: Float) -> Float {
        time % self.orbital_period / self.orbital_period * Float::TAU + self.mean_anomaly_at_epoch
    }

    /// Gets the distance from the central body at a given time
    #[allow(dead_code)] // Will be used in future
    fn get_radius(&self, mean_anomaly: Float) -> Float {
        self.semi_major_axis * (1.0 - self.eccentricity.powi(2))
            / (1.0 + self.eccentricity * mean_anomaly.cos())
    }

    /// Approximates the eccentric anomaly using fixed point iteration, should be within ±0.00005 radians.
    fn get_eccentric_anomaly(&self, mean_anomaly: Float) -> Float {
        let mut result = mean_anomaly;
        for _ in 0..20 {
            result = mean_anomaly + self.eccentricity * result.sin();
        }

        result
    }
}

#[typetag::serde]
impl Dynamic for Keplerian {
    /// Returns the offset from the parent body at a given time.
    fn get_offset(&self, time: crate::Float) -> Vector3<crate::Float> {
        let eccentric_anomaly = self.get_eccentric_anomaly(self.get_mean_anomaly(time));
        let (sin, cos) = eccentric_anomaly.sin_cos();
        // Top down view
        let x = self.semi_major_axis * (cos - self.eccentricity);
        let z = self.semi_major_axis * (1.0 - self.eccentricity.powi(2)).sqrt() * sin;

        // Convert to 3d by rotating around the `longitude of the ascending node` by `inclination`
        // radians
        let location = [x, 0.0, z];
        quaternion::rotate_vector(self.inclination, location).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IntermediateKeplerian {
    /// eccentricity
    e: Float,
    /// Semi-major axis (Maximum diameter of orbit)
    #[serde(rename = "a")]
    semimajor_axis: Float,

    /// inclination from the reference plane
    #[serde(rename = "i")]
    inclination: Float,
    /// Location where orbit intersects the reference plane from below to above
    #[serde(rename = "ascendingNode")]
    longitude_of_ascending_node: Float,

    /// Anomaly at T=0
    true_anomaly: Float,
    /// Location of periapsis relative to a reference point
    #[serde(rename = "argPeri")]
    argument_of_periapsis: Float,

    /// Time to complete one orbit, in hours
    period: Float,
}

impl From<IntermediateKeplerian> for Keplerian {
    fn from(value: IntermediateKeplerian) -> Self {
        Keplerian::new_with_period(
            value.e,
            value.semimajor_axis,
            value.inclination,
            value.longitude_of_ascending_node,
            value.argument_of_periapsis,
            value.true_anomaly,
            value.period,
        )
    }
}
impl From<Keplerian> for IntermediateKeplerian {
    fn from(value: Keplerian) -> Self {
        IntermediateKeplerian {
            e: value.eccentricity,
            semimajor_axis: value.semi_major_axis,
            inclination: value.calculated_fields.inclination,
            argument_of_periapsis: value.calculated_fields.argument_of_periapsis,
            longitude_of_ascending_node: value.calculated_fields.longitude_of_ascending_node,
            true_anomaly: value.mean_anomaly_at_epoch,
            period: value.orbital_period,
        }
    }
}

#[cfg(test)]
#[allow(clippy::excessive_precision)] // Tests should pass for f64 builds as well
mod tests {

    use super::*;

    fn get_earth() -> Keplerian {
        // from https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
        Keplerian::new_with_period(
            0.016_710_22,
            1.000_000_11 * 499.004_839,
            (0.000_05 as Float).to_radians(),
            (-11.260_64 as Float).to_radians(),
            (102.947_19 as Float).to_radians(),
            (100.464_35 as Float).to_radians(),
            365.256_36 * 24.0,
        )
    }

    #[test]
    fn orbital_period_from_parent_mass() {
        const PARENT_MASS: Float = 1048.0; // Mass of the sun
        const CHILD_MASS: Float = 0.003_146; // Mass of the earth
        const SEMI_MAJOR_AXIS: Float = 499.004_839; // Semi-major axis of the earth
        const EXPECTED: Float = 8_766.152_5;
        let orbit = Keplerian::new(
            0.0,
            SEMI_MAJOR_AXIS,
            0.0,
            0.0,
            0.0,
            0.0,
            PARENT_MASS + CHILD_MASS,
        );

        println!(
            "{} - {} = {}",
            orbit.orbital_period,
            EXPECTED,
            (orbit.orbital_period - EXPECTED).abs() / EXPECTED
        );

        // Get within 0.00005% of the "true" value (note: we aren't taking into account general
        // relativity, so it should always underestimate the time required)
        assert!((orbit.orbital_period - EXPECTED).abs() / EXPECTED < 5e-6);
    }
    #[test]
    fn anomaly_at_epoch() {
        let earth = get_earth();

        let anomaly = earth.get_mean_anomaly(0.0);

        assert!((anomaly - (100.464_35 as Float).to_radians()).abs() < 0.000_1);
    }

    #[test]
    fn anomaly_at_three_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomaly(earth.orbital_period / 4.0);

        assert!((anomaly - (190.464_35 as Float).to_radians()).abs() < 0.000_1);
    }

    #[test]
    fn anomaly_at_six_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomaly(earth.orbital_period / 2.0);

        assert!((anomaly - (280.464_35 as Float).to_radians()).abs() < 0.000_1);
    }

    fn get_tau_period() -> Keplerian {
        Keplerian::new_with_period(
            0.0,
            1.0,
            Float::FRAC_PI_2,
            Float::FRAC_PI_2,
            Float::FRAC_PI_2,
            0.0,
            Float::TAU,
        )
    }

    #[test]
    fn high_inclination() {
        // Start at the ascending node, go up then down
        let tau_period = Keplerian::new_with_period(
            0.0,
            1.0,
            Float::FRAC_PI_2,
            Float::FRAC_PI_2,
            0.0,
            Float::FRAC_PI_2,
            Float::TAU,
        );

        for i in 0_u8..100 {
            let theta = Float::from(i) / 100.0 * Float::TAU;

            let location = tau_period.get_offset(theta);
            print!(
                "time: {:.2}, location: ({:.2}, {:.4}, {:.2}), expected location: {:.4}",
                theta,
                location.x,
                location.y,
                location.z,
                theta.sin()
            );
            assert!((location.x - theta.sin()).abs() < 0.000_1);
            println!("\tSuccess ✅");
        }
    }

    #[test]
    /// The mean anomaly and the eccentric anomaly should always be equal when there is zero
    /// eccentricity
    fn zero_eccentricity() {
        let tau_period = get_tau_period();

        for i in 0..u8::MAX {
            let time = Float::from(i);
            let mean_anomaly = tau_period.get_mean_anomaly(time);
            assert!(
                (mean_anomaly - tau_period.get_eccentric_anomaly(mean_anomaly)).abs()
                    < Float::EPSILON
            );
        }
    }
}
