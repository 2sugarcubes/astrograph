use crate::{consts::GRAVITATIONAL_CONSTANT, Float};
use coordinates::prelude::*;
use quaternion::Quaternion;

use super::Dynamic;

pub struct Keplerian {
    // Size and shape
    eccentricity: Float,
    semi_major_axis: Float,
    semi_minor_axis: Float,

    // Orbital Plane, and argument of argument of periapsis
    inclination: Quaternion<Float>,

    //argument_of_periapsis: Float,
    mean_anomality_at_epoch: Float,

    orbital_period: Float,
}

impl Keplerian {
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

    #[must_use]
    pub fn new_with_period(
        eccentricity: Float,
        semi_major_axis: Float,
        inclination: Float,
        longitude_of_ascending_node: Float,
        argument_of_periapsis: Float,
        mean_anomality_at_epoch: Float,
        orbital_period: Float,
    ) -> Self {
        let semi_minor_axis = 1.0 - eccentricity * eccentricity;
        let inclination = quaternion::euler_angles(0.0, longitude_of_ascending_node, inclination);
        let inclination = quaternion::mul(
            inclination,
            quaternion::axis_angle(
                [0.0, 1.0, 0.0],
                argument_of_periapsis + longitude_of_ascending_node,
            ),
        );

        Self {
            eccentricity,
            semi_major_axis,
            semi_minor_axis,
            inclination,
            mean_anomality_at_epoch,
            orbital_period,
        }
    }

    /// Calculates the mean anomaly from the time since the epoch
    /// Note: May be larger than Tau, but should be fine since it will be used in sin or cos
    /// functions
    fn get_mean_anomality(&self, time: Float) -> Float {
        time % self.orbital_period / self.orbital_period * Float::TAU + self.mean_anomality_at_epoch
    }

    /// Gets the distance from the central body at a given time
    #[allow(dead_code)] // Will be used in future
    fn get_radius(&self, mean_anomality: Float) -> Float {
        self.semi_major_axis * self.semi_minor_axis
            / (1.0 + self.eccentricity * mean_anomality.cos())
    }

    /// Aproximates the eccentric anomaly using fixed point itteration, should be within ±0.00005 radians.
    fn get_eccentric_anomality(&self, mean_anomality: Float) -> Float {
        let mut result = mean_anomality;
        for _ in 0..20 {
            result = mean_anomality + self.eccentricity * result.sin();
        }

        result
    }
}

impl Dynamic for Keplerian {
    fn get_offset(&self, time: crate::Float) -> Vector3<crate::Float> {
        let eccentric_anomaly = self.get_eccentric_anomality(self.get_mean_anomality(time));
        let (sin, cos) = eccentric_anomaly.sin_cos();
        // Top down view
        let x = self.semi_major_axis * (cos - self.eccentricity);
        let z = self.semi_major_axis * (self.semi_minor_axis).sqrt() * sin;

        // Convert to 3d by rotating around the `longitude of the ascending node` by `inclination`
        // radians
        let location = [x, 0.0, z];
        quaternion::rotate_vector(self.inclination, location).into()
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
            1.000_000_11 * 499.0,
            (0.000_05 as Float).to_radians(),
            (-11.260_64 as Float).to_radians(),
            (102.947_19 as Float).to_radians(),
            (100.464_35 as Float).to_radians(),
            365.256 * 24.0,
        )
    }

    #[test]
    fn anomaly_at_epoch() {
        let earth = get_earth();

        let anomaly = earth.get_mean_anomality(0.0);

        assert!((anomaly - (100.46435 as Float).to_radians()).abs() < 0.000_1);
    }

    #[test]
    fn anomaly_at_three_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomality(earth.orbital_period / 4.0);

        assert!((anomaly - (190.46435 as Float).to_radians()).abs() < 0.000_1);
    }

    #[test]
    fn anomaly_at_six_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomality(earth.orbital_period / 2.0);

        assert!((anomaly - (280.46435 as Float).to_radians()).abs() < 0.000_1);
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
            assert!((location.x - theta.sin()).abs() < 0.0001);
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
            let mean_anomaly = tau_period.get_mean_anomality(time);
            assert!(
                (mean_anomaly - tau_period.get_eccentric_anomality(mean_anomaly)).abs()
                    < Float::EPSILON
            );
        }
    }
}
