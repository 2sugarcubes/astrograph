use super::{dynamic, float, random_angle, Arc, Body, Float, Planet, PlanetType};

/// Type of moon
#[derive(Debug, Clone)]
enum MoonType {
    /// Major Rocky moon e.g. Luna
    MajorRocky,
    /// Minor Rocky moon e.g. Deimos
    MinorRocky,
    /// Major Icy moon e.g. Europa
    MajorIcy,
    /// Minor Icy moon e.g. most bodies in saturn's rings
    MinorIcy,
}

#[derive(Debug, Clone)]
pub(super) struct Moon {
    /// Radius of the moon in ls
    radius: Float,
    /// Mass of the moon in jupiter masses
    mass: Float,
    /// Semi-major axis of the orbit in ls
    semi_major_axis: Float,
    /// Kind of the moon, e.g. minor icy
    kind: MoonType,
}

impl Moon {
    /// Earth's moon's density in jupiter masses per cubic light second
    const LUNA_DENSITY: Float = 47.47;

    /// # Returns
    /// None if all available orbits are already taken
    pub(super) fn new_moon<G: rand::Rng>(
        rng: &mut G,
        is_major: bool,
        is_icy: bool,
        parent: &Planet,
        hill_sphere_limit: Float,
        existing_moons: &[Self],
    ) -> Option<Self> {
        let density = if is_icy {
            // 1-2 g/cm^3
            rng.gen_range(14.195..28.39)
        } else {
            Self::LUNA_DENSITY * rng.gen_range(0.95..1.05)
        };

        let (hill_limit, radius, kind) = if is_major {
            let radius = rng.gen_range(0.001_001..parent.radius * 0.75);

            // If this moon is orbiting a terrestrial planet then divide the maximum semi-major axis
            // by two
            match parent.kind {
                PlanetType::GasGiant => (
                    hill_sphere_limit,
                    radius,
                    if is_icy {
                        MoonType::MajorIcy
                    } else {
                        MoonType::MajorRocky
                    },
                ),
                PlanetType::Habitable | PlanetType::Terestrial => (
                    hill_sphere_limit / 2.0,
                    radius,
                    if is_icy {
                        MoonType::MajorIcy
                    } else {
                        MoonType::MajorRocky
                    },
                ),
            }
        } else {
            // 200 to 300 km but in ls
            let radius = rng.gen_range(200.0..300.0) * 3.336e-6;
            (
                hill_sphere_limit,
                radius,
                if is_icy {
                    MoonType::MinorIcy
                } else {
                    MoonType::MinorRocky
                },
            )
        };
        let mass = radius.powi(3) * float::FRAC_2_PI * 2.0 / 3.0 * density;
        // Add some wriggle room since eccentricity will not be less than 0.001
        let roche_limit = radius * (2.0 * parent.mass / mass).cbrt() / 0.996;

        // Set the lower bound so that if the moon has eccentricity 0.5 it will still not pass
        // inside the roche limit
        let range =
            (roche_limit)..(hill_limit - existing_moons.len() as Float * parent.radius * 20.0);
        if range.is_empty() {
            return None;
        }
        let mut distance = rng.gen_range(range);

        let mut sorted_moons = Vec::new();
        sorted_moons.extend(existing_moons.iter());
        // Since there should not be any NaN or infinite values we should be in the clear
        sorted_moons.sort_by_key(|m| m.semi_major_axis.abs().to_bits());
        for m in sorted_moons {
            // If we are using some space required by another moon, shift our orbit
            // Note: this includes orbits well below our orbit, this is intentional to ensure
            // uniformity of random outcomes
            if m.semi_major_axis - parent.radius * 10.0 < distance {
                distance += parent.radius * 20.0;
            }
        }

        Some(Self {
            semi_major_axis: distance,
            mass,
            radius,
            kind,
        })
    }

    /// Generate "group a moons" for a gas giant
    pub(super) fn new_group_a_moons<G: rand::Rng>(rng: &mut G, parent: &Planet) -> Vec<Self> {
        let mut result = Vec::new();

        let mut semi_major_axis = (1.97 + rng.gen_range(-0.2..0.2)) * parent.radius;
        while semi_major_axis - 2.0 * 0.01861 < 2.44 * parent.radius {
            let radius: Float = rng.gen_range(20.0..200.0) * 3.336e-6;
            let mass = radius.powi(3) * float::FRAC_2_PI * 2.0 / 3.0
                * Self::LUNA_DENSITY
                * rng.gen_range(0.95..1.05);

            result.push(Self {
                semi_major_axis,
                radius,
                mass,
                kind: MoonType::MinorRocky,
            });

            semi_major_axis += rng.gen_range(0.00532..0.0319);
        }

        result
    }

    /// Generate "group b moons" for a gas giant
    pub(super) fn new_group_b_moons<G: rand::Rng>(rng: &mut G, parent: &Planet) -> Vec<Self> {
        let mut result = Vec::new();

        let mut semi_major_axis = 3.0 * parent.radius;

        while semi_major_axis <= 15.0 * parent.radius {
            let is_icy = rng.gen_bool(0.333);
            let min_mass = (0.001_001 as Float).powi(3) * float::FRAC_2_PI * 2.0 / 3.0
                * if is_icy { 21.3 } else { Self::LUNA_DENSITY };
            let mass = rng.gen_range(min_mass..0.0001 * parent.mass);
            let radius = (mass / (float::FRAC_2_PI * 2.0 / 3.0 * Self::LUNA_DENSITY)).cbrt();

            result.push(Self {
                semi_major_axis,
                mass,
                radius,
                kind: if is_icy {
                    MoonType::MajorIcy
                } else {
                    MoonType::MajorRocky
                },
            });
            semi_major_axis += rng.gen_range(parent.radius..5.0 * parent.radius);
        }
        result
    }

    /// Convert this moon to a body
    pub(super) fn to_body<G: rand::Rng>(
        &self,
        rng: &mut G,
        parent: &Planet,
        parent_body: &Arc,
        hill_sphere_limit: Float,
    ) -> Arc {
        let roche_limit = self.radius * (2.0 * parent.mass / self.mass).cbrt();
        let (inclination, eccentricity) = match self.kind {
            MoonType::MinorIcy | MoonType::MinorRocky => (
                rng.gen_range(-5.0 as Float..5.0).to_radians(),
                rng.gen_range(0.0..0.08),
            ),
            MoonType::MajorIcy | MoonType::MajorRocky => {
                // Clamp bounds to sensible values

                let eccentricity_range = match parent.kind {
                    PlanetType::GasGiant => 0.001..0.5,
                    PlanetType::Habitable | PlanetType::Terestrial => {
                        // Max eccentricity before hitting roche limit
                        let bound_a: Float = 1.0 - roche_limit / self.semi_major_axis;
                        // Max eccentricity before hitting hill sphere limit
                        let bound_b: Float = hill_sphere_limit / self.semi_major_axis - 1.0;

                        0.001..bound_a.min(bound_b).min(0.5)
                    }
                };
                (
                    rng.gen_range(0.0..float::FRAC_PI_2),
                    rng.gen_range(eccentricity_range),
                )
            }
        };
        Body::new(
            Some(parent_body.clone()),
            dynamic::keplerian::Keplerian::new(
                eccentricity,
                self.semi_major_axis,
                inclination + parent.north_pole.polar_angle,
                parent.north_pole.azimuthal_angle
                    + float::FRAC_PI_2
                    + rng.gen_range(-10.0 as Float..10.0).to_radians(),
                random_angle(rng),
                random_angle(rng),
                parent.mass,
            ),
        )
    }
}
