use crate::projection;

use super::{svg::Svg, Output};

use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
pub struct Web {
    svg: Svg<projection::StatelessOrthographic>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn draw_observation(time: u64, observations: String);
}

impl Output for Web {
    fn write_observations(
        &self,
        observations: &[crate::Observation],
        observatory_name: &str,
        time: i128,
        output_path_root: &Path,
    ) -> Result<(), std::io::Error> {
        let observations = self
            .svg
            .consume_observation(&format!("{time}"), observations);

        draw_observation(time, observations.to_string());

        Ok(())
    }
}

impl Default for Web {
    fn default() -> Self {
        Web {
            svg: Svg::new(projection::StatelessOrthographic()),
        }
    }
}
