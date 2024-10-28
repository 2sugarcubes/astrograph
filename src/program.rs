use std::path::PathBuf;

use derive_builder::Builder;

use crate::body::{observatory::Observatory, Arc};

/// A facade that takes values from [crate::body::observatory::Observatory] in the tree defined at the root of [`Self::_root_body`] that outputs using the given [outputers](crate::output::Output) provided with a [path](Self::output_file_root)
#[derive(Builder, Clone, Debug)]
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
    outputs: Vec<Box<dyn crate::output::Output>>,
    /// Location where output files will be stored, typically under a subdirectory for which
    /// observatory made that observation.
    output_file_root: PathBuf,
}

impl Program {
    /// Generate observations between the start and end time i.e. `[start_time, end_time)`, with
    /// observations every step_size hours.
    ///
    /// # Outputs
    ///
    /// Outputs depend on the implemtations of [`crate::output::Output`] used, but generally they
    /// will be files in the directory [`Self::output_file_root`]`/[OBSERVATORY NAME]/`
    // Precision loss is enevitable since we are going from an intager to a (compile-time) variable length float
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::missing_panics_doc)] // Should only panic in unit tests
    pub fn make_observations(&self, start_time: i128, end_time: i128, step_size: Option<usize>) {
        for time in (start_time..end_time).step_by(step_size.unwrap_or(1)) {
            for observatory in &self.observatories {
                let path = self
                    .output_file_root
                    // TODO real names
                    .join(format!("TODO OBSERVATORY NAME/{time:010}"));
                let observations = observatory.observe(time as f32);
                for output in &self.outputs {
                    // Write the observations to file, recovering on errors
                    match output.write_observations_to_file(&observations, &path) {
                        Ok(()) => println!(
                            "File {} was written sucessfully",
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
    }
}
