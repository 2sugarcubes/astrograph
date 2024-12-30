// TODO break functionality into different modules e.g. star, planet, moon

use std::ops::Range;

use coordinates::prelude::{Cylindrical, ThreeDimensionalConsts, Vector3};
use rich_progress_bar::RichProgressBar;

use crate::{
    body::{self, rotating::Rotating, Arc, Body},
    consts::float,
    dynamic::{self, fixed::Fixed, keplerian},
    Float,
};

use super::Generator;

pub struct Artifexian {}

impl Generator for Artifexian {
    fn generate<G: rand::Rng>(rng: &mut G) -> crate::body::Arc {
        let mut progress = RichProgressBar::new();

        progress
            .set_total(1_000)
            .set_color(rich_progress_bar::Colors::Blue)
            .set_progress_character('#')
            .set_bar_length(100);

        let root = Body::new(None, Fixed::new(Vector3::ORIGIN));
        for i in 0..1_000 {
            let _ = progress.inc();
            // At least 1% of stars are habitable
            let mut star = if i % 100 == 0 {
                MainSequenceStar::new_habitable(rng)
            } else {
                MainSequenceStar::new(rng)
            };

            //println!("Star mass: {}", star.mass);
            //println!(
            //    "Planet zone: {}..{}",
            //    star.planetary_zone.start, star.planetary_zone.end
            //);
            let first_gas_giant = Planet::new_from_frost_line(rng, &star);
            let mut planets = vec![first_gas_giant.clone()];

            let mut distance = (&first_gas_giant).semi_major_axis * rng.gen_range(1.4..2.0);
            while star.planetary_zone.contains(&distance) {
                planets.push(Planet::new_gas_giant(rng, distance));

                distance *= rng.gen_range(1.4..2.0);
            }
            let total_gas_giants = planets.len();
            //println!("Added {total_gas_giants} gas giants");

            distance = (&first_gas_giant).semi_major_axis / rng.gen_range(1.4..2.0);
            // If we have a habitable planet to add
            if let Some(habitable_planet) = Planet::new_habitable(rng, &star) {
                // We have a habitable planet to add
                let mut has_added_habitable_planet = false;
                let habitable_zone = (&habitable_planet).semi_major_axis / 1.4
                    ..(&habitable_planet).semi_major_axis * 1.4;

                // While we can add a planet
                while star.planetary_zone.contains(&distance) {
                    // If adding a planet would not be too close to the habitable planet
                    if (habitable_zone).contains(&distance) {
                        // Planet is too close to the habitable planet, so skip it
                        planets.push(habitable_planet.clone());
                        distance = (&habitable_planet).semi_major_axis;
                        has_added_habitable_planet = true;
                    } else if distance < (&habitable_planet).semi_major_axis
                        && !has_added_habitable_planet
                    {
                        // The next planet isn't too close to the habitable planet
                        planets.push(habitable_planet.clone());
                        planets.push(Planet::new_terrestial(rng, distance));
                        has_added_habitable_planet = true;
                    } else {
                        planets.push(Planet::new_terrestial(rng, distance));
                    }

                    // TODO break when distance between bodies is less than 0.15
                    distance /= rng.gen_range(1.4..2.0);
                }
            } else {
                // We don't have a habitable planet to add
                while star.planetary_zone.contains(&distance) {
                    planets.push(Planet::new_terrestial(rng, distance));

                    // TODO break when distance between bodies is less than 0.15
                    distance /= rng.gen_range(1.4..2.0);
                }
            }

            let mut filtered_planets = Vec::with_capacity(planets.len());

            planets.sort_by_key(|p| p.semi_major_axis.to_bits());

            let mut previous_planet = &planets[0];

            filtered_planets.push(previous_planet.clone());

            for i in 1..planets.len() {
                if previous_planet.semi_major_axis < &planets[i].semi_major_axis - au_to_ls(0.15) {
                    filtered_planets.push(planets[i].clone());
                    previous_planet = &planets[i];
                }
            }

            star.planets = filtered_planets;

            star.to_body(rng, &root);
        }
        return root;
    }
}

fn au_to_ls(au: Float) -> Float {
    au * 499.0
}

fn solar_masses_to_jupiter_masses(sm: Float) -> Float {
    sm * 1048.0
}

fn earth_masses_to_jupiter_masses(em: Float) -> Float {
    em * 0.003146
}

fn earth_radii_to_ls(er: Float) -> Float {
    er * 0.021251398
}

fn random_angle<G: rand::Rng>(rng: &mut G) -> Float {
    rng.gen_range(0.0..float::TAU)
}

#[derive(Debug, Clone)]
struct MainSequenceStar {
    mass: Float,
    //luminosity: Float,
    //diameter: Float,
    //surface_temp: Float,
    habitable_zone: Range<Float>,
    planetary_zone: Range<Float>,
    frost_line: Float,
    is_habitable: bool,
    planets: Vec<Planet>,
}

impl MainSequenceStar {
    fn new<G: rand::Rng>(rng: &mut G) -> Self {
        let mass: Float = rng.gen_range(0.02..16.0);
        Self::new_from_mass(mass)
    }

    fn new_habitable<G: rand::Rng>(rng: &mut G) -> Self {
        let mass: Float = rng.gen_range(0.6..1.4);
        Self::new_from_mass(mass)
    }

    /// # Note
    /// Mass is in solar masses, not jupiter masses as used throughout the rest of this library
    fn new_from_mass(mass: Float) -> Self {
        let luminosity = mass.powi(3);
        let sqrt_luminosity = luminosity.sqrt();
        Self {
            mass: solar_masses_to_jupiter_masses(mass),
            //luminosity,
            //diameter: mass.powf(0.74),
            //surface_temp: mass.powf(0.505),
            habitable_zone: au_to_ls(sqrt_luminosity * 0.95)..au_to_ls(sqrt_luminosity * 1.37),
            planetary_zone: au_to_ls(0.1 * mass)..au_to_ls(40.0 * mass),
            frost_line: au_to_ls(4.85 * sqrt_luminosity),
            is_habitable: (0.6..1.4).contains(&mass),
            planets: Vec::new(),
        }
    }

    fn allowed_height(radius: Float) -> Float {
        const SIGMA: Float = 40963.2174964452;
        const SIGMA_SQUARED: Float = SIGMA * SIGMA;
        let maximum =
            (2600.0 / (float::TAU.sqrt() * SIGMA)).powf(radius * radius / (2.0 * SIGMA_SQUARED));
        //println!("Got allowed height {}", maximum * 1.029e8);
        // Convert pc to ls
        maximum * 1.029e8
    }

    fn to_body<G: rand::Rng>(&self, rng: &mut G, root: &Arc) -> Arc {
        const WIDTH_OF_MILKY_WAY: Float = 3e12;

        //println!("Generating star position");
        let d = rand_distr::Normal::new(0.0, 0.44).unwrap();

        //println!("Calculating radius");
        let radius = (rng.sample(d) * WIDTH_OF_MILKY_WAY).abs();
        //println!("Calculating height");
        let height = rng.sample(d) * Self::allowed_height(radius);
        //println!("Calculating theta");
        let theta = float::TAU // Convert revs to radians
            * (if rng.gen() {
                // The primary arm
                rng.sample(d) * 0.25
            } else {
                // Make a second arm, half a turn from the primary
                rng.sample(d)* 0.25 + 0.5
            } + radius / (WIDTH_OF_MILKY_WAY * 1.5)); // Make theta map out one and a half turns on
                                                      // the way from the centre to the outer rim

        // Use fixed as a performance saver since their periods would be on the order of millions
        // of years
        //println!("Packing star to body");
        let b = Body::new(
            Some(root.clone()),
            dynamic::fixed::Fixed(Cylindrical::new(radius, height, theta).into()),
        );

        //println!("Adding bodies");
        // Add planets to this body
        for p in &self.planets {
            p.to_body(rng, self, &b);
        }

        return b;
    }
}

#[derive(Debug, Clone)]
struct Planet {
    semi_major_axis: Float,
    mass: Float,
    radius: Float,
    planet_type: PlanetType,
}

#[derive(Debug, Clone)]
enum PlanetType {
    Terestrial,
    GasGiant,
    Habitable,
}

impl Planet {
    fn new_from_frost_line<G: rand::Rng>(rng: &mut G, parent_star: &MainSequenceStar) -> Self {
        let semi_major_axis = parent_star.frost_line + au_to_ls(rng.gen_range(1.0..1.2));

        let (mass, radius) = Self::generate_gas_giant_perameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            planet_type: PlanetType::GasGiant,
        }
    }

    fn new_habitable<G: rand::Rng>(rng: &mut G, parent_star: &MainSequenceStar) -> Option<Self> {
        if parent_star.is_habitable {
            let semi_major_axis = rng.gen_range(parent_star.habitable_zone.clone());
            let (mass, radius) = Self::generate_terestial_parameters(rng);

            Some(Self {
                semi_major_axis,
                mass,
                radius,
                planet_type: PlanetType::Habitable,
            })
        } else {
            None
        }
    }

    fn new_terrestial<G: rand::Rng>(rng: &mut G, semi_major_axis: Float) -> Self {
        let (mass, radius) = Self::generate_terestial_parameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            planet_type: PlanetType::Terestrial,
        }
    }

    fn new_gas_giant<G: rand::Rng>(rng: &mut G, semi_major_axis: Float) -> Self {
        let (mass, radius) = Self::generate_gas_giant_perameters(rng);

        Self {
            semi_major_axis,
            mass,
            radius,
            planet_type: PlanetType::GasGiant,
        }
    }

    fn max_terestrial_moons(&self, planetary_zone_end: Float) -> (u8, u8) {
        let minor_moons = match self.planet_type {
            PlanetType::GasGiant => {
                todo!()
            }
            PlanetType::Habitable | PlanetType::Terestrial => {
                let x = self.semi_major_axis / planetary_zone_end;
                ((2.0 as Float).powf(x) * x * 6.0).floor() as u8
            }
        };

        let major_moons = match self.planet_type {
            PlanetType::Terestrial => 0,
            PlanetType::Habitable => 1,
            PlanetType::GasGiant => todo!(),
        };

        (major_moons, minor_moons)
    }

    fn generate_gas_giant_perameters<G: rand::Rng>(rng: &mut G) -> (Float, Float) {
        let mass = rng.gen_range(earth_masses_to_jupiter_masses(10.0)..13.0);
        let radius = 0.2333
            * if mass >= 2.0 {
                rng.gen_range(0.98..1.02)
            } else {
                rng.gen_range(1.0..1.9)
            };

        (mass, radius)
    }

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

    fn to_body<G: rand::Rng>(
        &self,
        rng: &mut G,
        parent_star: &MainSequenceStar,
        parent: &Arc,
    ) -> Arc {
        let dynamic = match self.planet_type {
            PlanetType::GasGiant => keplerian::Keplerian::new(
                rng.gen_range(0.001..0.1),
                self.semi_major_axis,
                Float::to_radians(rng.gen_range(0.0..4.0)),
                random_angle(rng),
                random_angle(rng),
                random_angle(rng),
                parent_star.mass,
            ),
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
                    0.000_01,
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
                    rng.gen_range(0.0..10.0),
                    random_angle(rng),
                    random_angle(rng),
                    random_angle(rng),
                    parent_star.mass,
                )
            }
        };

        let hill_sphere_limit = &dynamic.semi_major_axis
            * (1.0 - &dynamic.eccentricity)
            * (self.mass / (3.0 * (self.mass + parent_star.mass))).cbrt();
        let b = body::Body::new(Some(parent.clone()), dynamic);
        for m in self.generate_moons(rng, parent_star, hill_sphere_limit) {
            m.to_body(rng, self, &b, hill_sphere_limit);
        }

        match self.planet_type {
            PlanetType::Habitable => {
                // Put some rotation on it
                b.write().unwrap().rotation = Some(Rotating::new(
                    // 12 to 36 hour rotation speed
                    rng.gen_range(12.0..36.0),
                    coordinates::prelude::Spherical {
                        radius: 1.0,
                        polar_angle: (rng.gen_range(0.0..80.0) as Float
                            // Make it rotate retrograde 20% of the time
                            + if rng.gen_bool(0.2) { 90.0 } else { 0.0 })
                        .to_radians(),
                        azimuthal_angle: random_angle(rng),
                    },
                ));
            }
            _ => (),
        }
        b
    }
    fn generate_moons<G: rand::Rng>(
        &self,
        rng: &mut G,
        star: &MainSequenceStar,
        hill_sphere_limit: Float,
    ) -> Vec<Moon> {
        let mut moons = Vec::new();

        match self.planet_type {
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

#[derive(Debug, Clone)]
enum MoonType {
    MajorRocky,
    MinorRocky,
    MajorIcy,
    MinorIcy,
}

#[derive(Debug, Clone)]
struct Moon {
    radius: Float,
    mass: Float,
    semi_major_axis: Float,
    moon_type: MoonType,
}

impl Moon {
    // Earth's moon's density in jupiter masses per cubic light second
    const LUNA_DENSITY: Float = 47.47;

    /// # Returns
    /// None if all available orbits are already taken
    fn new_moon<G: rand::Rng>(
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

        let (hill_limit, radius, moon_type) = if is_major {
            let radius = rng.gen_range(0.001001..parent.radius * 0.75);

            // If this moon is orbiting a terrestial planet then divide the maximum semi-major axis
            // by two
            match parent.planet_type {
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
        let roche_limit = radius * (2.0 * parent.mass / mass).cbrt();

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
            moon_type,
        })
    }

    fn new_group_a_moons<G: rand::Rng>(rng: &mut G, parent: &Planet) -> Vec<Self> {
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
                moon_type: MoonType::MinorRocky,
            });

            semi_major_axis += rng.gen_range(0.00532..0.0319);
        }

        result
    }

    fn new_group_b_moons<G: rand::Rng>(rng: &mut G, parent: &Planet) -> Vec<Self> {
        let mut result = Vec::new();

        let mut semi_major_axis = 3.0 * parent.radius;

        while semi_major_axis <= 15.0 * parent.radius {
            let is_icy = rng.gen_bool(0.333);
            let min_mass = (0.001001 as Float).powi(3) * float::FRAC_2_PI * 2.0 / 3.0
                * if is_icy { 21.3 } else { Self::LUNA_DENSITY };
            let mass = rng.gen_range(min_mass..0.0001 * parent.mass);
            let radius = (mass / (float::FRAC_2_PI * 2.0 / 3.0 * Self::LUNA_DENSITY)).cbrt();

            result.push(Self {
                semi_major_axis,
                mass,
                radius,
                moon_type: if is_icy {
                    MoonType::MajorIcy
                } else {
                    MoonType::MajorRocky
                },
            });
            semi_major_axis += rng.gen_range(parent.radius..5.0 * parent.radius);
        }
        result
    }

    fn to_body<G: rand::Rng>(
        &self,
        rng: &mut G,
        parent: &Planet,
        parent_body: &Arc,
        hill_sphere_limit: Float,
    ) -> Arc {
        let roche_limit = self.radius * (2.0 * parent.mass / self.mass).cbrt();
        let (inclination, eccentricity) = match self.moon_type {
            MoonType::MinorIcy | MoonType::MinorRocky => (
                rng.gen_range(-5.0 as Float..5.0).to_radians(),
                rng.gen_range(0.0..0.08),
            ),
            MoonType::MajorIcy | MoonType::MajorRocky => {
                // Clamp bounds to sensable values

                let eccentricity_range = match parent.planet_type {
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
                inclination,
                random_angle(rng),
                random_angle(rng),
                random_angle(rng),
                parent.mass,
            ),
        )
    }
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn flame_test() {
        //let mut rng = rand::rngs::mock::StepRng::new(0, 1);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(42_123);
        let _ = Artifexian::generate(&mut rng);
        const INC: u64 = 0x0101_0101_0101_0101;
        let mut rng = rand::rngs::mock::StepRng::new(INC, INC);
        let _ = Artifexian::generate(&mut rng);
    }
}
