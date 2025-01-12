use crate::{
    body::{
        observatory::{self, Observatory, WeakObservatory},
        Arc,
    },
    generator::{artifexian::Artifexian, Generator},
    output::web::Web,
    program::ProgramBuilder,
};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use wasm_bindgen::prelude::*;

/// # Errors
/// Reutrns an error if root or observatories are not valid representations of their values i.e. missing
/// required fields
#[wasm_bindgen]
pub fn generate_observations_from_json(
    root: JsValue,
    observatories: JsValue,
    start_time: i128,
    end_time: i128,
    step_size: Option<usize>,
) -> Result<(), JsError> {
    // Create root body (and whole body tree)
    let root: Arc = serde_wasm_bindgen::from_value(root)?;

    // Create weak observatories to avoid memory duplication
    let observatories: Vec<WeakObservatory> = serde_wasm_bindgen::from_value(observatories)?;

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
    serde_wasm_bindgen::to_value(&Artifexian::generate(&mut XorShiftRng::seed_from_u64(seed)))
        .unwrap()
}

#[wasm_bindgen]
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should not be able to panic
pub fn generate_universe() -> JsValue {
    serde_wasm_bindgen::to_value(&Artifexian::generate(&mut XorShiftRng::from_entropy())).unwrap()
}
