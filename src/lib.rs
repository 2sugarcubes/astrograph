// Style guide suggests using return for any function over 5 lines to improve readability
#![allow(clippy::needless_return)]

pub mod body;
#[allow(clippy::excessive_precision)] // Constants should work with up to f128 precision
pub mod consts;
pub mod dynamic;
pub mod output;
pub mod program;
pub mod projection;

#[allow(unused_imports)] // Macro_use is required here
#[macro_use]
#[cfg(test)]
extern crate assert_float_eq;

type Float = f32;

pub mod testing {
    use xorshift::{Rng, SeedableRng, Xorshift128};

    use crate::{
        body::{Arc, Body},
        consts::float,
        dynamic::keplerian::Keplerian,
        Float,
    };

    pub const DEFAULT_SEED: u64 = 0x064B_DEAF_BEEF_CAFE;

    #[must_use]
    pub fn make_toy_example(seed: u64) -> (Arc, Arc) {
        let mut rng = Xorshift128::from_seed(&[seed, seed]);

        let (root, observer) = make_toy_parents(&mut rng, 5);

        make_toy_children(&mut rng, &observer, 2, 9);

        (root, observer)
    }

    #[must_use]
    #[allow(clippy::cast_lossless)] // Necesary to enable testing on multiple float types
    fn make_toy_parents<T: Rng>(rng: &mut T, depth: u8) -> (Arc, Arc) {
        if depth == 0 {
            let body = Body::new(None, make_keplernian_dynamic(rng, -(depth as i32)));
            (body.clone(), body)
        } else {
            let (root, parent) = make_toy_parents(rng, depth - 1);
            let body = Body::new(Some(parent), make_keplernian_dynamic(rng, -(depth as i32)));
            (root, body)
        }
    }

    #[allow(clippy::cast_lossless)]
    fn make_toy_children<T: Rng>(rng: &mut T, parent: &Arc, depth: u8, number_of_children: u8) {
        if depth >= 1 {
            for _ in 0..number_of_children {
                let child = Body::new(
                    Some(parent.clone()),
                    make_keplernian_dynamic(rng, -(depth as i32)),
                );
                make_toy_children(rng, &child, depth - 1, number_of_children);
            }
        }
    }

    #[allow(clippy::useless_conversion)] // Necesary for testing with different length floats
    fn make_keplernian_dynamic<T: Rng>(rng: &mut T, depth: i32) -> Keplerian {
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

    fn make_random_angle<T: Rng>(rng: &mut T) -> Float {
        rng.gen_range(0.0, float::TAU)
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use xorshift::{thread_rng, Xoroshiro128, Xorshift1024};

        #[test]
        fn right_number_of_bodies() {
            const DEPTH_OF_CHILDREN: u32 = 3;
            const NUMBER_OF_CHILDREN: u32 = 4;
            const NUMBER_OF_PARENTS: u32 = 5;
            let mut rng = Xoroshiro128::from_seed(&[DEFAULT_SEED, DEFAULT_SEED]);
            let (root, observer) = make_toy_parents(&mut rng, NUMBER_OF_PARENTS as u8);

            // We add one so that we are also counting the observer body
            assert_eq!(count_bodies(root.clone()), NUMBER_OF_PARENTS + 1);
            make_toy_children(
                &mut rng,
                &observer,
                DEPTH_OF_CHILDREN as u8,
                NUMBER_OF_CHILDREN as u8,
            );

            let mut expected_children = 0;
            for depth in 1..=DEPTH_OF_CHILDREN {
                expected_children += NUMBER_OF_CHILDREN.pow(depth)
            }

            assert_eq!(
                count_bodies(root),
                NUMBER_OF_PARENTS + 1 + expected_children
            );
        }

        fn count_bodies(body: Arc) -> u32 {
            // Start at one to count this body
            let mut count = 1;
            for child in &body.read().unwrap().children {
                // Count the children and their children
                count += count_bodies(child.to_owned())
            }
            return count;
        }

        #[test]
        fn fuzz_toy_examples() {
            let mut rng: Xorshift1024 = thread_rng();

            for _ in 0..5_000 {
                let seed = rng.gen();
                println!("Seed was: {:x?}", seed);
                let _ = make_toy_example(seed);
            }
        }
    }
}
