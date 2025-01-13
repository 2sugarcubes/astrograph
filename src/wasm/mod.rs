use std::sync::{Arc, RwLock, Weak};

use crate::{
    body::{
        self,
        observatory::{self, Observatory, WeakObservatory},
        rotating::Rotating,
    },
    dynamic::{fixed::Fixed, keplerian::Keplerian},
    generator::{artifexian::Artifexian, Generator},
    output::web::Web,
    program::ProgramBuilder,
};
use gloo_utils::format::JsValueSerdeExt;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
    let root = upgrade_body(&fake_root, None);

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
        ._root_body(root)
        .outputs(vec![Box::new(Web::default())])
        .observatories(observatories)
        .build()?;

    program.make_observations(start_time, end_time, step_size);
    Ok(())
}

#[wasm_bindgen]
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should not be able to panic
pub fn generate_universe_from_seed(seed: u64) -> JsValue {
    JsValue::from_serde(&Artifexian::generate(&mut XorShiftRng::seed_from_u64(seed))).unwrap()
}

#[wasm_bindgen]
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should not be able to panic
pub fn generate_universe() -> JsValue {
    JsValue::from_serde(&Artifexian::generate(&mut XorShiftRng::from_entropy())).unwrap()
}

/// A kind of hacky solution to the problem of serde json not recognising typetaged dynamics when
/// targeting a wasm arch
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    /// Bodies that orbit around this body
    children: Vec<Box<Body>>,
    /// The way this body moves around the parent
    dynamic: Dynamic,
    /// If the body has any o1fservatories it is highly recommended to initialize this.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    rotation: Option<Rotating>,
    // Getting some parameters ready for a next version
    // /// Mass of the body in jupiter masses
    //mass: Float,
    //radius: Float,
    //color: [u8,h8,u8],
}

#[derive(Clone, Debug, Deserialize)]
enum Dynamic {
    Fixed(Fixed),
    Keplerian(Keplerian),
}

fn upgrade_body(body: &self::Body, parent: Option<Arc<RwLock<body::Body>>>) -> body::Arc {
    let dynamic: Box<dyn crate::dynamic::Dynamic> = match body.dynamic {
        Dynamic::Fixed(f) => Box::new(f),
        Dynamic::Keplerian(k) => Box::new(k),
    };

    let result = Arc::new(RwLock::new(body::Body {
        parent: parent.map(|p| Arc::downgrade(&p)),
        dynamic,
        rotation: body.rotation.clone(),
        children: Vec::with_capacity(body.children.len()),
    }));

    if let Ok(mut lock) = result.write() {
        for c in &body.children {
            let child = upgrade_body(c, Some(result.clone()));
            lock.children.push(child);
        }
    }

    result
}
