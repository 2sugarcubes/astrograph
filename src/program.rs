use std::path::PathBuf;

use derive_builder::Builder;

use crate::body::{observatory::Observatory, Arc};

#[derive(Builder, Clone, Debug)]
pub struct Program {
    _root_body: Arc,
    #[builder(setter(each(name = "add_observatory")))]
    observatories: Vec<Observatory>,
    #[builder(setter(each(name = "add_output")))]
    outputs: Vec<Box<dyn crate::output::Output>>,
    output_file_root: PathBuf,
}

impl Program {
    /// Generate observations between the start and end time i.e. `[start_time, end_time)`, with
    /// observations every step_size hours.
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
