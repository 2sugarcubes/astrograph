use super::{
    au_to_ls, body, earth_masses_to_jupiter_masses, earth_radii_to_ls, float, keplerian,
    random_angle, Arc, Float, MainSequenceStar, Moon, Rotating, Spherical, ThreeDimensionalConsts,
    Vector3,
};

/// A planet that orbits a star
#[derive(Debug, Clone)]
pub(super) struct Planet {
    /// Semi-major axis of this body's orbit in ls
    pub(super) semi_major_axis: Float,
    /// mass of this planet in jupiter masses
    pub(super) mass: Float,
    /// Radius of this body in ls
    pub(super) radius: Float,
    /// Type of this body, e.g. Habitable, Terestrial, or Gas Giant.
    pub(super) kind: PlanetType,
    /// The location of true-north so we can generate a rotation for this body
    pub(super) north_pole: Spherical<Float>,
}

/// Planet types
#[derive(Debug, Clone)]
pub(super) enum PlanetType {
    /// A terrestrial planet e.g. Mars
    Terestrial,
    /// A gas giant, e.g. Jupiter
    GasGiant,
    /// A Habitable planet, e.g. Earth
    Habitable,
}

impl Planet {
    /// Generate a gas giant based on the frost line of the star
    pub(super) fn new_from_frost_line<G: rand::Rng>(
        rng: &mut G,
        parent_star: &MainSequenceStar,
    ) -> Self {
        let semi_major_axis = parent_star.frost_line + au_to_ls(rng.gen_range(1.0..1.2));

        let (mass, radius) = Self::generate_gas_giant_parameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            kind: PlanetType::GasGiant,
            north_pole: Spherical::new(1.0, random_angle(rng), random_angle(rng)),
        }
    }

    /// Generate a habitable planet from the habitable zone of a star
    pub(super) fn new_habitable<G: rand::Rng>(
        rng: &mut G,
        parent_star: &MainSequenceStar,
    ) -> Option<Self> {
        if parent_star.is_habitable {
            const UP: Vector3<Float> = Vector3::<Float>::UP;
            let sma_range =
                parent_star.habitable_zone.start / 0.996..parent_star.habitable_zone.end / 1.003;
            let semi_major_axis = rng.gen_range(sma_range);
            let (mass, radius) = Self::generate_terestial_parameters(rng);

            let mut north_pole: Vector3<Float> = Spherical::new(
                1.0,
                rng.gen_range(-80.0 as Float..80.0).to_radians()
                    + if rng.gen_bool(0.1) { float::PI } else { 0.0 },
                random_angle(rng),
            )
            .into();

            let star_north_pole: Vector3<Float> = parent_star.north_pole.into();
            let rotation = quaternion::rotation_from_to(UP.into(), star_north_pole.into());

            north_pole = quaternion::rotate_vector(rotation, north_pole.into()).into();

            Some(Self {
                semi_major_axis,
                mass,
                radius,
                kind: PlanetType::Habitable,
                north_pole: north_pole.into(),
            })
        } else {
            None
        }
    }

    /// Generate a Terestrial planet based on the given semi-major axis
    pub(super) fn new_terrestrial<G: rand::Rng>(rng: &mut G, semi_major_axis: Float) -> Self {
        let (mass, radius) = Self::generate_terestial_parameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            kind: PlanetType::Terestrial,
            north_pole: Spherical::new(1.0, random_angle(rng), random_angle(rng)),
        }
    }

    /// Generate a gas giant based on the given semi-major axis
    pub(super) fn new_gas_giant<G: rand::Rng>(rng: &mut G, semi_major_axis: Float) -> Self {
        let (mass, radius) = Self::generate_gas_giant_parameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            kind: PlanetType::GasGiant,
            north_pole: Spherical::new(1.0, random_angle(rng), random_angle(rng)),
        }
    }

    /// Calculate how many major and minor moons a Terestrial planet should have
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn max_terestrial_moons(&self, planetary_zone_end: Float) -> (u8, u8) {
        let minor_moons = match self.kind {
            PlanetType::GasGiant => 0,
            PlanetType::Habitable | PlanetType::Terestrial => {
                let x = self.semi_major_axis / planetary_zone_end;
                ((2.0 as Float).powf(x) * x * 6.0).floor() as u8
            }
        };

        let major_moons = match self.kind {
            PlanetType::Terestrial => 0,
            PlanetType::Habitable => 1,
            PlanetType::GasGiant => todo!(),
        };

        (major_moons, minor_moons)
    }

    /// Generates the mass and radius of a gas giant
    fn generate_gas_giant_parameters<G: rand::Rng>(rng: &mut G) -> (Float, Float) {
        let mass = rng.gen_range(earth_masses_to_jupiter_masses(10.0)..13.0);
        let radius = 0.2333
            * if mass >= 2.0 {
                rng.gen_range(0.98..1.02)
            } else {
                rng.gen_range(1.0..1.9)
            };

        (mass, radius)
    }

    /// Generates the mass and radius of a terrestrial planet
    fn generate_terestial_parameters<G: rand::Rng>(rng: &mut G) -> (Float, Float) {
        // Terestrial
        let mass: Float = rng.gen_range(0.18..3.5);
        // Clamp radius to make surface gravity logical
        let radius: Float =
            mass * rng.gen_range((0.4 / mass).sqrt().max(0.5)..(1.6 / mass).sqrt().min(1.5));

        (
            earth_masses_to_jupiter_masses(mass),
            earth_radii_to_ls(radius),
        )
    }

    /// Converts a planet to a body that can be added to the tree
    pub(super) fn to_body<G: rand::Rng>(
        &self,
        rng: &mut G,
        parent_star: &MainSequenceStar,
        parent: &Arc,
    ) -> Arc {
        let longitude_of_ascending_node = parent_star.north_pole.azimuthal_angle
            + float::FRAC_PI_2
            + rng.gen_range(-float::FRAC_PI_8..float::FRAC_PI_8) / 2.0;

        let inclination =
            parent_star.north_pole.polar_angle + rng.gen_range(-10.0 as Float..10.0).to_radians();

        let dynamic = match self.kind {
            PlanetType::GasGiant => {
                let inclination = parent_star.north_pole.polar_angle
                    + rng.gen_range(-4.0 as Float..4.0).to_radians();
                keplerian::Keplerian::new(
                    rng.gen_range(0.001..0.1),
                    self.semi_major_axis,
                    inclination,
                    longitude_of_ascending_node,
                    random_angle(rng),
                    random_angle(rng),
                    parent_star.mass,
                )
            }
            PlanetType::Habitable => {
                // Habitable world
                // Make sure the eccentricity will not take us out of the habitable zone AT ALL
                let bound_a = 1.0 - parent_star.habitable_zone.start / self.semi_major_axis;
                let bound_b = parent_star.habitable_zone.end / self.semi_major_axis - 1.0;

                let range = 0.00001..bound_a.min(bound_b).min(0.2);

                let eccentricity = rng.gen_range(range);
                keplerian::Keplerian::new(
                    eccentricity,
                    self.semi_major_axis,
                    inclination,
                    random_angle(rng),
                    random_angle(rng),
                    random_angle(rng),
                    parent_star.mass,
                )
            }
            PlanetType::Terestrial =>
            // Mercury, Venus, or Mars like for example
            {
                keplerian::Keplerian::new(
                    rng.gen_range(0.0..0.25),
                    self.semi_major_axis,
                    inclination,
                    random_angle(rng),
                    random_angle(rng),
                    random_angle(rng),
                    parent_star.mass,
                )
            }
        };

        let hill_sphere_limit = dynamic.semi_major_axis
            * (1.0 - &dynamic.eccentricity)
            * (self.mass / (3.0 * (self.mass + parent_star.mass))).cbrt();
        let b = body::Body::new(Some(parent.clone()), dynamic);
        for m in self.generate_moons(rng, parent_star, hill_sphere_limit) {
            m.to_body(rng, self, &b, hill_sphere_limit);
        }

        if let PlanetType::Habitable = self.kind {
            // Put some rotation on it
            b.write().unwrap().rotation = Some(Rotating::new(
                // 12 to 36 hour rotation speed
                rng.gen_range(12.0..36.0),
                coordinates::prelude::Spherical {
                    radius: 1.0,
                    polar_angle: (rng.gen_range(0.0..80.0) as Float
                        // Make it rotate retrograde 20% of the time
                        + if rng.gen_bool(0.2) { 100.0 } else { 0.0 })
                    .to_radians(),
                    azimuthal_angle: random_angle(rng),
                },
            ));
        }
        b
    }

    /// Generates the moons around this planet
    fn generate_moons<G: rand::Rng>(
        &self,
        rng: &mut G,
        star: &MainSequenceStar,
        hill_sphere_limit: Float,
    ) -> Vec<Moon> {
        let mut moons = Vec::new();

        match self.kind {
            PlanetType::GasGiant => {
                moons.extend(Moon::new_group_a_moons(rng, self));
                moons.extend(Moon::new_group_b_moons(rng, self));
            }
            PlanetType::Terestrial => {
                let (max_minor_moons, max_major_moons) =
                    self.max_terestrial_moons(star.planetary_zone.end);
                for _ in 0..max_major_moons {
                    let is_icy = rng.gen_bool(0.1);
                    if let Some(new_moon) =
                        Moon::new_moon(rng, true, is_icy, self, hill_sphere_limit, &moons)
                    {
                        moons.push(new_moon);
                    } else {
                        // Moon orbits are full
                        break;
                    }
                }
                for _ in 0..max_minor_moons {
                    let is_icy = rng.gen_bool(0.1);
                    if let Some(new_moon) =
                        Moon::new_moon(rng, true, is_icy, self, hill_sphere_limit, &moons)
                    {
                        moons.push(new_moon);
                    } else {
                        // Moon orbits are full
                        break;
                    }
                }
            }
            PlanetType::Habitable => {
                let (max_minor_moons, max_major_moons) =
                    self.max_terestrial_moons(star.planetary_zone.end);
                // Have at least one major moon
                for _ in 0..max_major_moons.min(1) {
                    let is_icy = rng.gen_bool(0.1);
                    if let Some(new_moon) =
                        Moon::new_moon(rng, true, is_icy, self, hill_sphere_limit, &moons)
                    {
                        moons.push(new_moon);
                    } else {
                        // Moon orbits are full
                        break;
                    }
                }
                for _ in 0..max_minor_moons {
                    let is_icy = rng.gen_bool(0.1);
                    if let Some(new_moon) =
                        Moon::new_moon(rng, true, is_icy, self, hill_sphere_limit, &moons)
                    {
                        moons.push(new_moon);
                    } else {
                        // Moon orbits are full
                        break;
                    }
                }
            }
        }
        moons
    }
}
