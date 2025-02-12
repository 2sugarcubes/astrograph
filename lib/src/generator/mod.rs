use rand::Rng;

use crate::body::observatory::Observatory;
use crate::body::Arc;

pub mod artifexian;

pub trait Generator {
    /// Generates stars, planets, and moons based on settings made to self --- e.g. number of stars ---
    /// and the random number generator given. As well as a liist of observatories that were
    /// generated on planets
    fn generate<G: Rng>(&self, rng: &mut G) -> (Arc, Vec<Observatory>);
}
