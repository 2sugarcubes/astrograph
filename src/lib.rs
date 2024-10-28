// Style guide suggests using return for any function over 5 lines to improve readability
#![allow(clippy::needless_return)]

/// Structures that model discrete objects in the simulation, e.g. planets, stars, and
/// observatories.
pub mod body;

/// Physical constants for the simulation, e.g. The Gravitational Constant, Pi, and Tau.
#[allow(clippy::excessive_precision)] // Constants should work with up to f128 precision
pub mod consts;
/// Structs that model the orbits that bodies can follow.
pub mod dynamic;
/// Objects that assist in outputting data to various types, e.g. HTML canvas, SVG, etc.
pub mod output;
/// A helper [facade](https://en.wikipedia.org/wiki/Facade_pattern) that takes simulation times and
/// converts them to outputs, such as SVG files.
pub mod program;

/// Projections that map 3D space to a 2D plane
pub mod projection;

#[allow(unused_imports)] // Macro_use is required here
#[macro_use]
#[cfg(test)]
extern crate assert_float_eq;

/// Type alias to enable compile time configurable precision.
#[cfg(not(feature = "f64"))]
pub type Float = f32;

#[cfg(feature = "f64")]
pub type Float = f64;

/// Useful functions to use while testing to cut down on code repetition.
pub mod testing {
    use xorshift::{Rng, SeedableRng, Xorshift128};

    use crate::{
        body::{Arc, Body},
        consts::float,
        dynamic::keplerian::Keplerian,
        Float,
    };

    pub const DEFAULT_SEED: u64 = 0x064B_DEAF_BEEF_CAFE;

    /// Generates an example [crate::body::Body] tree from a seed, at the moment this will only
    /// generate a tree with five ancestors of the observing body, the observing body, nine
    /// children of the observing body, and 9 children of those children (81 descendants in total)
    ///
    /// # Returns
    ///
    /// `([crate::body::Arc], [crate::body::Arc])` where the first `Arc` is the root body, and must
    /// be kept alive to keep all the ancestors of the intended observing body in scope; and the second `Arc` is intended to be the observing body.
    #[must_use]
    pub fn make_toy_example(seed: u64) -> (Arc, Arc) {
        let mut rng = Xorshift128::from_seed(&[seed, seed]);

        let (root, observer) = make_toy_parents(&mut rng, 5);

        make_toy_children(&mut rng, &observer, 2, 9);

        (root, observer)
    }

    /// Generates the ancestors of the intended observer along with the observer
    ///
    /// # Returns
    /// `([crate::body::Arc], [crate::body::Arc])` where the first `Arc` is the root body, and must
    /// be kept alive to keep all the ancestors of the intended observing body in scope; and the second `Arc` is intended to be the observing body.
    #[must_use]
    #[allow(clippy::cast_lossless)] // Necessary to enable testing on multiple float types
    fn make_toy_parents<T: Rng>(rng: &mut T, depth: u8) -> (Arc, Arc) {
        if depth == 0 {
            let body = Body::new(None, make_keplerian_dynamic(rng, -(depth as i32)));
            (body.clone(), body)
        } else {
            let (root, parent) = make_toy_parents(rng, depth - 1);
            let body = Body::new(Some(parent), make_keplerian_dynamic(rng, -(depth as i32)));
            (root, body)
        }
    }

    /// Generates the children of some body by mutating that body.
    #[allow(clippy::cast_lossless)]
    fn make_toy_children<T: Rng>(rng: &mut T, parent: &Arc, depth: u8, number_of_children: u8) {
        if depth >= 1 {
            for _ in 0..number_of_children {
                let child = Body::new(
                    Some(parent.clone()),
                    make_keplerian_dynamic(rng, -(depth as i32)),
                );
                make_toy_children(rng, &child, depth - 1, number_of_children);
            }
        }
    }

    /// Generates a random plausible [crate::dynamic::keplerian::Keplerian] where depth acts as a
    /// scaling factor where each step deeper halves the size of the variables that it generates
    /// (`semi_major_axis` and `parent_mass`)
    #[allow(clippy::useless_conversion)] // Necessary for testing with different length floats
    fn make_keplerian_dynamic<T: Rng>(rng: &mut T, depth: i32) -> Keplerian {
        let scale = (2.0 as Float).powi(depth.into());

        Keplerian::new(
            rng.gen_range(0.01, 1.0),
            rng.gen_range(200.0, 19_700.0) * scale,
            make_random_angle(rng),
            make_random_angle(rng),
            make_random_angle(rng),
            make_random_angle(rng),
            rng.gen_range(0.95, 1.05) * scale,
        )
    }

    /// Convenience wrapper that generates a random canonical angle, i.e. [0, 2π), in radians
    fn make_random_angle<T: Rng>(rng: &mut T) -> Float {
        rng.gen_range(0.0, float::TAU)
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use xorshift::{thread_rng, Xoroshiro128, Xorshift1024};

        #[test]
        #[allow(clippy::cast_lossless)]
        fn right_number_of_bodies() {
            const DEPTH_OF_CHILDREN: u8 = 3;
            const NUMBER_OF_CHILDREN: u8 = 4;
            const NUMBER_OF_PARENTS: u8 = 5;
            let mut rng = Xoroshiro128::from_seed(&[DEFAULT_SEED, DEFAULT_SEED]);
            let (root, observer) = make_toy_parents(&mut rng, NUMBER_OF_PARENTS);

            // We add one so that we are also counting the observer body
            assert_eq!(count_bodies(&root), NUMBER_OF_PARENTS as u32 + 1);
            make_toy_children(&mut rng, &observer, DEPTH_OF_CHILDREN, NUMBER_OF_CHILDREN);

            let mut expected_children = 0_u32;
            for depth in 1..=DEPTH_OF_CHILDREN {
                expected_children += (NUMBER_OF_CHILDREN as u32).pow(depth as u32);
            }

            assert_eq!(
                count_bodies(&root),
                NUMBER_OF_PARENTS as u32 + 1 + expected_children
            );
        }

        fn count_bodies(body: &Arc) -> u32 {
            // Start at one to count this body
            let mut count = 1;
            for child in &body.read().unwrap().children {
                // Count the children and their children
                count += count_bodies(child);
            }
            return count;
        }

        #[test]
        fn fuzz_toy_examples() {
            let mut rng: Xorshift1024 = thread_rng();

            for _ in 0..5_000 {
                let seed = rng.gen();
                println!("Seed was: {seed:x?}");
                let _ = make_toy_example(seed);
            }
        }
    }
}
