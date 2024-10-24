use std::path::PathBuf;

use crate::body::{observatory::Observatory, ArcBody};

// TODO make a builder type for this struct
pub struct Program<OutputType> {
    _root_body: ArcBody,
    observatories: Vec<Observatory>,
    outputs: Vec<Box<dyn crate::output::Output<OutType = OutputType>>>,
    output_file_root: PathBuf,
}

impl<T> Program<T>
where
    T: Sized,
{
    pub fn make_observations(&self, start_time: i128, end_time: i128, step_size: Option<usize>) {
        for time in (start_time..end_time).step_by(step_size.unwrap_or(1)) {
            for observatory in &self.observatories {
                let path = self
                    .output_file_root
                    // TODO real names
                    .join(format!("TODO OBSERVATORY NAME/{time}"));
                let observations = observatory.observe(time as f32);
                for output in &self.outputs {
                    // generate file contents
                    let file_contents = output.consume_observation(&observations);

                    // Write the file, recovering on errors
                    match output.write_to_file(file_contents, &path) {
                        Ok(_) => println!(
                            "File {} was written sucessfully",
                            &path.to_str().unwrap_or("[could not display path]")
                        ),
                        Err(e) => {
                            let message = format!(
                                "ERROR WRITING FILE/DIRECTORY {}, message: {e}",
                                &path.to_str().unwrap_or("[could not display path]")
                            );
                            if cfg!(test) {
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
