use rand::Rng;

use crate::body::Arc;

pub mod artifexian;

pub trait Generator {
    fn generate<G: Rng>(rng: &mut G) -> Arc;
}
