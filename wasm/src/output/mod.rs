use astrolabe::projection;

use astrolabe::output::{svg::Svg, Output};

use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
pub struct Web {
    svg: Svg<projection::StatelessOrthographic>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn draw_observation(time: i128, observations: String);
}

impl Output for Web {
    fn write_observations(
        &self,
        observations: &[astrolabe::LocalObservation],
        _observatory_name: &str,
        time: i128,
        _output_path_root: &std::path::Path,
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
