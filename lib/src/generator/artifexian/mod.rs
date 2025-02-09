use std::ops::Range;

use coordinates::prelude::{Cylindrical, Spherical, ThreeDimensionalConsts, Vector3};
use derive_builder::Builder;

use crate::{
    body::{self, rotating::Rotating, Arc, Body},
    consts::float,
    dynamic::{self, fixed::Fixed, keplerian},
    Float,
};

use super::Generator;
use moon::Moon;
use planet::Planet;
use planet::PlanetType;
use star::MainSequenceStar;

/// Logic for generating bodies that orbit planets
mod moon;
/// Logic for generating bodies that orbit stars
mod planet;
/// Logic for generating bodies that are "fixed" in space (Have a very long period that would
/// appear fixed over short observational periods <100 years)
mod star;

#[derive(Clone, Copy, Debug, Builder, PartialEq, Eq, PartialOrd, Ord)]
pub struct Artifexian {
    /// Number of stars to generate
    #[builder(default = 1_000_000)]
    star_count: usize,
}

impl Generator for Artifexian {
    /// Generates bodies based on the star count and random number generator. Generated observatories are on "habitable" worlds.
    fn generate<G: rand::Rng>(
        &self,
        rng: &mut G,
    ) -> (crate::body::Arc, Vec<crate::body::observatory::Observatory>) {
        let root = Body::new(None, Fixed::new(Vector3::ORIGIN));

        let mut observatories = Vec::with_capacity(self.star_count / 100);

        for i in 0..self.star_count {
            // At least 1% of stars are habitable
            let star = if i % 100 != 0 {
                // Skip planet gen to save memory
                MainSequenceStar::new(rng)
            } else {
                // Habitable star, so generate planets
                let mut star = MainSequenceStar::new_habitable(rng);
                let first_gas_giant = Planet::new_from_frost_line(rng, &star);
                let mut planets = vec![first_gas_giant.clone()];

                let mut distance = first_gas_giant.semi_major_axis * rng.gen_range(1.4..2.0);
                while star.planetary_zone.contains(&distance) {
                    planets.push(Planet::new_gas_giant(rng, distance));

                    distance *= rng.gen_range(1.4..2.0);
                }

                distance = first_gas_giant.semi_major_axis / rng.gen_range(1.4..2.0);
                // If we have a habitable planet to add
                if let Some(habitable_planet) = Planet::new_habitable(rng, &star) {
                    // We have a habitable planet to add
                    let mut has_added_habitable_planet = false;
                    let habitable_zone = habitable_planet.semi_major_axis / 1.4
                        ..habitable_planet.semi_major_axis * 1.4;

                    // While we can add a planet
                    while star.planetary_zone.contains(&distance) {
                        // If adding a planet would not be too close to the habitable planet
                        if (habitable_zone).contains(&distance) {
                            // Planet is too close to the habitable planet, so skip it
                            planets.push(habitable_planet.clone());
                            distance = habitable_planet.semi_major_axis;
                            has_added_habitable_planet = true;
                        } else if distance < habitable_planet.semi_major_axis
                            && !has_added_habitable_planet
                        {
                            // The next planet isn't too close to the habitable planet
                            planets.push(habitable_planet.clone());
                            planets.push(Planet::new_terrestrial(rng, distance));
                            has_added_habitable_planet = true;
                        } else {
                            planets.push(Planet::new_terrestrial(rng, distance));
                        }

                        // TODO break when distance between bodies is less than 0.15
                        distance /= rng.gen_range(1.4..2.0);
                    }
                } else {
                    // We don't have a habitable planet to add
                    while star.planetary_zone.contains(&distance) {
                        planets.push(Planet::new_terrestrial(rng, distance));

                        // TODO break when distance between bodies is less than 0.15
                        distance /= rng.gen_range(1.4..2.0);
                    }
                }

                let mut filtered_planets = Vec::with_capacity(planets.len());

                planets.sort_by_key(|p| p.semi_major_axis.to_bits());

                let mut previous_planet = &planets[0];

                filtered_planets.push(previous_planet.clone());

                for i in planets.iter().skip(1) {
                    if previous_planet.semi_major_axis < i.semi_major_axis - au_to_ls(0.15) {
                        filtered_planets.push(i.clone());
                        previous_planet = i;
                    }
                }

                star.planets = filtered_planets;
                star
            };

            if let (_, Some(observer)) = star.to_body(rng, &root) {
                observatories.push(observer);
            }
        }
        return (root, observatories);
    }
}

/// Convert Astronomical Units (AU) to Light Seconds (ls)
fn au_to_ls(au: Float) -> Float {
    au * 499.0
}

/// Convert solar masses to jupiter masses
fn solar_masses_to_jupiter_masses(sm: Float) -> Float {
    sm * 1048.0
}

/// Convert earth masses to jupiter masses
fn earth_masses_to_jupiter_masses(em: Float) -> Float {
    em * 0.003_146
}

/// Convert Earth Radii to Light Seconds (ls)
fn earth_radii_to_ls(er: Float) -> Float {
    er * 0.021_251_398
}

/// Generate a random angle between 0 and Tau
fn random_angle<G: rand::Rng>(rng: &mut G) -> Float {
    rng.gen_range(0.0..float::TAU)
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use super::*;

    //#[ignore = "long running"]
    #[test]
    fn flame_test() {
        //TODO there seems to be a problem with distributions when inc contains more than 7
        //consecuive zeros
        const INC: u64 = 0x0101_0101_0101_0101;

        //let mut rng = rand::rngs::mock::StepRng::new(0, 1);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(42_123);
        let generator = ArtifexianBuilder::default()
            .star_count(1_000)
            .build()
            .unwrap();
        let root = generator.generate(&mut rng);

        println!("x\ty\tz");
        for p in &root.0.read().unwrap().children {
            let loc: Vector3<Float> = p.read().unwrap().dynamic.get_offset(0.0);
            println!("{}\t{}\t{}", loc.x, loc.y, loc.z);
        }

        drop(root);
        let mut rng = rand::rngs::mock::StepRng::new(INC + (INC >> 8) + (INC >> 16), INC);
        let _ = generator.generate(&mut rng);
    }
}
