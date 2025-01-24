use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use dyn_clone::DynClone;

use crate::LocalObservation;

/// An output for SVG files
pub mod svg;

pub mod logger;

/// An output for the website
#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
pub mod web;

/// The trait for structs that output to a file. It may be made more general in future to better
/// accommodate non-file outputs e.g. console loggers, or outputs to screen or streams
pub trait Output: DynClone + Debug {
    /// # Errors
    /// implementations may panic if there is an error in the filesystem e.g. writing is not
    /// allowed for a user in a specific directory, or one or more of the directories are files
    /// that have already been created
    fn write_observations(
        &self,
        observations: &[LocalObservation],
        observatory_name: &str,
        time: i128,
        output_path_root: &Path,
    ) -> Result<(), std::io::Error>;

    /// # Errors
    /// implementations may panif if there is an error in the filesystem e.g. the user is missing
    /// permissions, a directory in the path is a file.
    fn flush(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}
dyn_clone::clone_trait_object!(Output);

#[must_use]
pub fn to_default_path(
    output_path_root: &Path,
    observatory_name: &str,
    time: i128,
    extension: &str,
) -> PathBuf {
    let mut path = output_path_root.to_owned();
    path.push(observatory_name);
    path.push(format!("{time:010}{extension}"));

    path
}

#[cfg(test)]
mod tests {}
