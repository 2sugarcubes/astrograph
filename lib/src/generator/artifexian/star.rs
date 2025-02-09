use super::{
    au_to_ls, dynamic, float, random_angle, solar_masses_to_jupiter_masses, Arc, Body, Cylindrical,
    Float, Planet, Range, Spherical,
};

/// A star that can have bodies that orbit it
#[derive(Debug, Clone)]
pub(super) struct MainSequenceStar {
    /// Mass of the star in jupiter masses
    pub(super) mass: Float,
    //luminosity: Float,
    //diameter: Float,
    //surface_temp: Float,
    /// Area where habitable planets can exist in ls (light seconds)
    pub(super) habitable_zone: Range<Float>,
    /// Area where planets can exist
    pub(super) planetary_zone: Range<Float>,
    /// Line where all planets become gas giants or pluto like
    pub(super) frost_line: Float,
    /// If this star can support water based life in it's lifetime
    pub(super) is_habitable: bool,
    /// The location of true north above this star, defining the elliptical plane
    pub(super) north_pole: Spherical<Float>,
    /// List of planets that orbit this star
    pub(super) planets: Vec<Planet>,
}

impl MainSequenceStar {
    /// Generate a star that may or may not be habitable
    pub(super) fn new<G: rand::Rng>(rng: &mut G) -> Self {
        let mass: Float = rng.gen_range(0.02..16.0);
        Self::new_from_mass(rng, mass)
    }

    /// Generate a habitable star
    pub(super) fn new_habitable<G: rand::Rng>(rng: &mut G) -> Self {
        let mass: Float = rng.gen_range(0.6..1.4);
        Self::new_from_mass(rng, mass)
    }

    /// # Note
    /// Mass is in solar masses, not jupiter masses as used throughout the rest of this library
    fn new_from_mass<G: rand::Rng>(rng: &mut G, mass: Float) -> Self {
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
            north_pole: Spherical::new(1.0, random_angle(rng), random_angle(rng)),
            planets: Vec::new(),
        }
    }

    /// Gets the allowed deviation above or below the universal reference plane
    #[allow(clippy::excessive_precision)] // Needs to work for f32 and f64 versions
    fn allowed_height(radius: Float) -> Float {
        const SIGMA: Float = 40_963.217_496_445_2;
        const SIGMA_SQUARED: Float = SIGMA * SIGMA;
        let maximum =
            (2600.0 / (float::TAU.sqrt() * SIGMA)).powf(radius * radius / (2.0 * SIGMA_SQUARED));
        // Convert pc to ls
        maximum * 1.029e8
    }

    /// Convert this star to a body to add to the body tree
    pub(super) fn to_body<G: rand::Rng>(
        &self,
        rng: &mut G,
        root: &Arc,
    ) -> (Arc, Option<crate::body::observatory::Observatory>) {
        const WIDTH_OF_MILKY_WAY: Float = 3e12;

        let d = rand_distr::Pert::new(-1.0, 1.0, 0.0).unwrap();

        let radius = (rng.sample(d) * WIDTH_OF_MILKY_WAY).abs();
        let height = rng.sample(d) * Self::allowed_height(radius);
        let theta = if radius > 5e11 {
            float::TAU // Convert revs to radians
            * (if rng.gen() {
                // The primary arm
                rng.sample(d) * 0.25
            } else {
                // Make a second arm, half a turn from the primary
                rng.sample(d) * 0.25 + 0.5
            } + 1.0 + radius * 1.352 / (WIDTH_OF_MILKY_WAY)) // Make theta map out one and a half turns on
                                                             // the way from the center to the outer rim
        } else {
            random_angle(rng)
        };
        // Use fixed as a performance saver since their periods would be on the order of millions
        // of years
        let b = Body::new(
            Some(root.clone()),
            dynamic::fixed::Fixed(Cylindrical::new(radius, height, theta).into()),
        );

        let mut observatory = None;
        // Add planets to this body
        for p in &self.planets {
            let arc = p.to_body(rng, self, &b);

            match p.kind {
                super::planet::PlanetType::Habitable => {
                    use coordinates::prelude::*;
                    let name = match arc.read() {
                        Ok(b) => Err(b.get_id()),
                        Err(_) => Ok("Unnamed".to_string()),
                    };

                    observatory = Some(crate::body::observatory::Observatory::new(
                        Spherical::FORWARD,
                        arc,
                        name,
                    ));
                }
                super::planet::PlanetType::Terestrial | super::planet::PlanetType::GasGiant => (),
            }
        }

        return (b, observatory);
    }
}
