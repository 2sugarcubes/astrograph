use crate::{consts::GRAVITATIONAL_CONSTANT, FLOAT};
use coordinates::prelude::*;
use quaternion::Quaternion;

use super::Dynamic;

pub struct Keplerian {
    // Size and shape
    eccentricity: FLOAT,
    semi_major_axis: FLOAT,
    semi_minor_axis: FLOAT,

    // Orbital Plane, and argument of argument of periapsis
    inclination: Quaternion<FLOAT>,

    //argument_of_periapsis: FLOAT,
    mean_anomality_at_epoch: FLOAT,

    orbital_period: FLOAT,
}

impl Keplerian {
    pub fn new(
        eccentricity: FLOAT,
        semi_major_axis: FLOAT,
        inclination: FLOAT,
        longitude_of_ascending_node: FLOAT,
        argument_of_periapsis: FLOAT,
        true_anomaly: FLOAT,
        parent_mass: FLOAT,
    ) -> Self {
        let orbital_period = FLOAT::TAU
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

    pub fn new_with_period(
        eccentricity: FLOAT,
        semi_major_axis: FLOAT,
        inclination: FLOAT,
        longitude_of_ascending_node: FLOAT,
        argument_of_periapsis: FLOAT,
        mean_anomality_at_epoch: FLOAT,
        orbital_period: FLOAT,
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
    fn get_mean_anomality(&self, time: FLOAT) -> FLOAT {
        time % self.orbital_period / self.orbital_period * FLOAT::TAU + self.mean_anomality_at_epoch
    }

    /// Gets the distance from the central body at a given time
    fn get_radius(&self, mean_anomality: FLOAT) -> FLOAT {
        self.semi_major_axis * self.semi_minor_axis
            / (1.0 + self.eccentricity * mean_anomality.cos())
    }

    /// Aproximates the eccentric anomaly using fixed point itteration, should be within ±0.00005 radians.
    fn get_eccentric_anomality(&self, mean_anomality: FLOAT) -> FLOAT {
        let mut result = mean_anomality;
        for _ in 0..20 {
            result = mean_anomality + self.eccentricity * result.sin()
        }

        result
    }
}

impl Dynamic for Keplerian {
    fn get_offset(&self, time: crate::FLOAT) -> Vector3<crate::FLOAT> {
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
mod tests {

    use std::u8;

    use super::*;

    const DEG_TO_RAD: FLOAT = 0.017453292519943295769236907684886127134389;

    fn get_earth() -> Keplerian {
        // from https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
        Keplerian::new_with_period(
            0.01671022,
            1.00000011 * 499.0,
            0.00005 * DEG_TO_RAD,
            -11.26064 * DEG_TO_RAD,
            102.94719 * DEG_TO_RAD,
            100.46435 * DEG_TO_RAD,
            365.256 * 24.0,
        )
    }

    #[test]
    fn anomaly_at_epoch() {
        let earth = get_earth();

        let anomaly = earth.get_mean_anomality(0.0);

        assert!((anomaly - 100.46435 * DEG_TO_RAD).abs() < 0.0001);
    }

    #[test]
    fn anomaly_at_three_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomality(earth.orbital_period / 4.0);

        assert!((anomaly - 190.46435 * DEG_TO_RAD).abs() < 0.0001);
    }

    #[test]
    fn anomaly_at_six_months() {
        let earth = get_earth();
        let anomaly = earth.get_mean_anomality(earth.orbital_period / 2.0);

        assert!((anomaly - 280.46435 * DEG_TO_RAD).abs() < 0.0001);
    }

    fn get_tau_period() -> Keplerian {
        Keplerian::new_with_period(
            0.0,
            1.0,
            FLOAT::FRAC_PI_2,
            FLOAT::FRAC_PI_2,
            FLOAT::FRAC_PI_2,
            0.0,
            FLOAT::TAU,
        )
    }

    #[test]
    fn high_inclination() {
        // Start at the ascending node, go up then down
        let tau_period = Keplerian::new_with_period(
            0.0,
            1.0,
            FLOAT::FRAC_PI_2,
            FLOAT::FRAC_PI_2,
            0.0,
            FLOAT::FRAC_PI_2,
            FLOAT::TAU,
        );

        for i in 0_u8..100 {
            let theta = FLOAT::from(i) / 100.0 * FLOAT::TAU;

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
            println!("\tSuccess ✅")
        }
    }

    #[test]
    /// The mean anomaly and the eccentric anomaly should always be equal when there is zero
    /// eccentricity
    fn zero_eccentricity() {
        let tau_period = get_tau_period();

        for i in 0..u8::MAX {
            let time = FLOAT::from(i);
            let mean_anomaly = tau_period.get_mean_anomality(time);
            assert!(
                (mean_anomaly - tau_period.get_eccentric_anomality(mean_anomaly)).abs()
                    < FLOAT::EPSILON
            );
        }
    }
}
