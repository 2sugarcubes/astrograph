use astrograph::projection;

use astrograph::{
    constelation::Line,
    output::{svg::Svg, Output},
};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
pub struct Web {
    svg: Svg<projection::StatelessOrthographic>,
    // PERF: switch to a vector with an intelligent offset to speed up hashing since we know the
    // that observations will be within a set range
    observations: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<i128, svg::Document>>>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn draw_observation(time: i128, observations: String);
}

impl Output for Web {
    fn write_observations(
        &self,
        observations: &[astrograph::LocalObservation],
        constellations: &[Line],
        _observatory_name: &str,
        time: i128,
        _output_path_root: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let observations =
            self.svg
                .consume_observation(&format!("{time}"), observations, constellations);

        if let Ok(mut hash_map) = self.observations.write() {
            hash_map.insert(time, observations);
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        if let Ok(hash_map) = self.observations.read() {
            let mut observations: Vec<_> = hash_map.par_iter().collect();
            observations.par_sort_unstable_by_key(|x| x.0);
            for (time, svg) in observations {
                draw_observation(*time, svg.to_string());
            }
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Observations became poisoned",
            ))
        }
    }
}

impl Default for Web {
    fn default() -> Self {
        Web {
            svg: Svg::new(projection::StatelessOrthographic()),
            observations: std::sync::Arc::new(std::sync::RwLock::new(
                std::collections::HashMap::default(),
            )),
        }
    }
}
