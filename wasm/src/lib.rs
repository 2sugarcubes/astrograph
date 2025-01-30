use astrolabe::{
    body::{
        observatory::{self, Observatory, WeakObservatory},
        rotating::Rotating,
    },
    dynamic::{fixed::Fixed, keplerian::Keplerian},
    generator::{artifexian::ArtifexianBuilder, Generator},
    program::ProgramBuilder,
    Float,
};
use gloo_utils::format::JsValueSerdeExt;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use output::Web;
mod output;

/// # Errors
/// Reutrns an error if root or observatories are not valid representations of their values i.e. missing
/// required fields
#[wasm_bindgen]
pub fn generate_observations_from_json(
    root: &str,
    observatories: &str,
    start_time: i128,
    end_time: i128,
    step_size: Option<usize>,
) -> Result<(), JsError> {
    // Create root body (and whole body tree)
    let fake_root: self::Body = serde_json::from_str(root)?;
    let root = astrolabe::body::Body::try_from(fake_root).unwrap().into();

    astrolabe::body::Body::hydrate_all(&root, &None);

    // Create weak observatories to avoid memory duplication
    let observatories: Vec<WeakObservatory> = serde_json::from_str(observatories)?;

    // Upgrade weak observatories
    let observatories: Vec<Observatory> = observatories
        .into_iter()
        .map(|o| observatory::to_observatory(o, &root))
        .collect();

    // Avoid potential zero step size
    let step_size = step_size.filter(|x| *x != 0);

    // Create program that outputs to the page
    let program = ProgramBuilder::default()
        .root_body(root)
        .outputs(vec![Box::new(Web::default())])
        .observatories(observatories)
        .build()?;

    program.make_observations(start_time, end_time, step_size);
    Ok(())
}

#[cfg_attr(any(target_arch = "wasm32", target_arch = "wasm64"), wasm_bindgen)]
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should not be able to panic
pub fn generate_universe_from_seed(seed: u64) -> JsValue {
    JsValue::from_serde(
        &ArtifexianBuilder::default()
            .star_count(1_000_000)
            .build()
            .unwrap()
            .generate(&mut XorShiftRng::seed_from_u64(seed)),
    )
    .unwrap()
}

#[cfg_attr(any(target_arch = "wasm32", target_arch = "wasm64"), wasm_bindgen)]
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should not be able to panic
pub fn generate_universe() -> JsValue {
    JsValue::from_serde(
        &ArtifexianBuilder::default()
            .star_count(1_000_000)
            .build()
            .unwrap()
            .generate(&mut XorShiftRng::from_entropy()),
    )
    .unwrap()
}

/// A kind of hacky solution to the problem of serde json not recognising typetaged dynamics when
/// targeting a wasm arch
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    /// Bodies that orbit around this body
    children: Vec<Body>,
    /// The way this body moves around the parent
    dynamic: Dynamic,
    /// If the body has any o1fservatories it is highly recommended to initialize this.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    rotation: Option<Rotating>,
    // Getting some parameters ready for a next version
    // /// Mass of the body in jupiter masses
    //mass: Float,
    radius: Option<crate::Float>,
    //color: [u8,h8,u8],
    name: Option<String>,
}

impl TryFrom<Body> for astrolabe::body::Body {
    type Error = astrolabe::body::BodyBuilderError;
    fn try_from(value: Body) -> Result<Self, Self::Error> {
        astrolabe::body::BodyBuilder::default()
            .parent(None)
            .name(value.name.map_or(astrolabe::body::Name::Unknown, |n| {
                astrolabe::body::Name::Named(n.into())
            }))
            .children(
                value
                    .children
                    .into_iter()
                    .filter_map(|c| astrolabe::body::Body::try_from(c).ok().map(|b| b.into()))
                    .collect(),
            )
            .radius(value.radius)
            .rotation(value.rotation)
            .dynamic(match value.dynamic {
                Dynamic::Fixed(f) => Box::new(f),
                Dynamic::Keplerian(f) => Box::new(f),
            })
            .build()
    }
}

#[derive(Clone, Debug, Deserialize)]
enum Dynamic {
    Fixed(Fixed),
    Keplerian(Keplerian),
}
