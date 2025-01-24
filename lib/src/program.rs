use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    body::{
        observatory::{to_observatory, Observatory, WeakObservatory},
        Arc,
    },
    output::Output,
    Float,
};

/// A facade that takes values from [`crate::body::observatory::Observatory`] in the tree defined at the root of [`Self::_root_body`] that outputs using the given [outputs](crate::output::Output) provided with a [path](Self::output_file_root)
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "DeserializedProgram")]
pub struct Program {
    /// The root of the tree, we need to reference it here to prevent the reference counter from
    /// reaching zero prematurely.
    // TODO #[builder(setter(name = root_body))]
    _root_body: Arc,
    /// List of observatories that we need to observe from to.
    #[builder(setter(each(name = "add_observatory")))]
    observatories: Vec<Observatory>,
    /// List of outputs to use.
    #[builder(setter(each(name = "add_output")))]
    #[serde(skip)]
    outputs: Vec<Box<dyn crate::output::Output>>,
    /// Location where output files will be stored, typically under a subdirectory for which
    /// observatory made that observation.
    #[builder(default)]
    output_file_root: PathBuf,
}

impl Program {
    /// Generate observations between the start and end time i.e. `[start_time, end_time)`, with
    /// observations every `step_size` hours.
    ///
    /// # Outputs
    ///
    /// Outputs depend on the implementations of [`crate::output::Output`] used, but generally they
    /// will be files in the directory [`Self::output_file_root`]`/[OBSERVATORY NAME]/`
    // Precision loss is inevitable since we are going from an integer to a (compile-time) variable length float
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::missing_panics_doc)] // Should only panic in unit tests
    pub fn make_observations(&self, start_time: i128, end_time: i128, step_size: Option<usize>) {
        for time in (start_time..end_time).step_by(step_size.unwrap_or(1)) {
            for observatory in &self.observatories {
                let path = self
                    .output_file_root
                    // TODO real names
                    .join(format!("{}/{time:010}", observatory.get_name()));
                let observations = observatory.observe(time as Float);
                for output in &self.outputs {
                    // Write the observations to file, recovering on errors
                    match output.write_observations(
                        &observations,
                        &observatory.get_name(),
                        time,
                        &self.output_file_root,
                    ) {
                        Ok(()) => println!(
                            "File {} was written successfully",
                            &path.to_str().unwrap_or("[could not display path]")
                        ),
                        Err(e) => {
                            let message = format!(
                                "ERROR WRITING FILE/DIRECTORY {}, message: {e}",
                                &path.to_str().unwrap_or("[could not display path]")
                            );
                            if cfg!(test) {
                                // Panic can only occur in internal testing mode when panics are expected
                                panic!("{message}");
                            } else {
                                //TODO implement log or something similar
                                eprintln!("{message}");
                            }
                        }
                    }
                }
            }
        }

        for output in &self.outputs {
            match output.flush() {
                Ok(()) => (),
                Err(e) => eprintln!("{e}"),
            };
        }
    }

    /// Set the output root
    pub fn set_output_path<T: Into<PathBuf>>(&mut self, output: T) {
        self.output_file_root = output.into();
    }

    pub fn add_output(&mut self, output_method: Box<dyn Output>) {
        self.outputs.push(output_method);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeserializedProgram {
    root_body: Arc,
    observatories: Vec<WeakObservatory>,
    output_file_root: PathBuf,
}

impl From<DeserializedProgram> for Program {
    fn from(value: DeserializedProgram) -> Self {
        let mut observatories = Vec::with_capacity(value.observatories.len());

        for o in value.observatories {
            observatories.push(to_observatory(o, &value.root_body));
        }

        crate::body::Body::hydrate_all(&value.root_body, &None);

        Program {
            _root_body: value.root_body,
            observatories,
            output_file_root: value.output_file_root,
            outputs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_from_parts() {
        let bodies = include_str!("../../assets/solar-system.json");
        let observatoies_str = include_str!("../../assets/solar-system.observatories.json");

        let root: Arc = serde_json::from_str(bodies).unwrap();
        let observatories: Vec<WeakObservatory> = serde_json::from_str(observatoies_str).unwrap();

        let dp = DeserializedProgram {
            root_body: root.clone(),
            observatories,
            output_file_root: PathBuf::default(),
        };

        let program: Program = dp.into();

        assert_eq!(6, program.observatories.len());
    }

    #[test]
    fn deserialize() {
        let program = include_str!("../../assets/solar-system.program.json");

        let program: Program = serde_json::from_str(program).unwrap();

        assert_eq!(6, program.observatories.len());
    }
}
