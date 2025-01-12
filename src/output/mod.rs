use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use coordinates::prelude::Spherical;
use dyn_clone::DynClone;

use crate::{body::Arc, Float};

/// An output for SVG files
pub mod svg;

/// An output for the website
pub mod web;

/// The trait for structs that output to a file. It may be made more general in future to better
/// accommodate non-file outputs e.g. console loggers, or outputs to screen or streams
pub trait Output: DynClone + Debug {
    /// # Errors
    /// implementations may panic if there is an error in the filesystem e.g. writing is not
    /// allowed for a user in a specific directory, or one or more of the directories are files
    /// that have already been created
    fn write_observations_to_file(
        &self,
        observations: &[(Arc, Spherical<Float>)],
        path: &Path,
    ) -> Result<(), std::io::Error>;
}
dyn_clone::clone_trait_object!(Output);

/// Changes the extension of the given path, even when the last element is perceived to be a directory
fn set_extension(path: &Path, extension: &str) -> PathBuf {
    let mut path = path.to_path_buf();
    // Make sure Rust isn't trying to be clever and take our file without an extension and convert
    // it to a directory
    if path.is_dir() {
        path.pop();
    }
    path.set_extension(extension);
    return path;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::output::set_extension;

    #[test]
    fn set_extension_for_file_without_extension() {
        let path = set_extension(&PathBuf::from("path/to/file"), "txt");
        assert!(
            path.eq(&PathBuf::from("path/to/file.txt")),
            "Path {} does not match path 'path/to/file.txt'",
            path.to_str().unwrap_or("[COULD NOT DISPLAY]")
        );
    }

    #[test]
    fn set_extension_for_file_with_extension() {
        let path = set_extension(&PathBuf::from("path/to/file.bin"), "txt");
        assert!(
            path.eq(&PathBuf::from("path/to/file.txt")),
            "Path {} does not match path 'path/to/file.txt'",
            path.to_str().unwrap_or("[COULD NOT DISPLAY]")
        );
    }

    #[test]
    fn set_extension_for_directory() {
        let path = set_extension(&PathBuf::from("path/to/directory/".to_string()), "txt");
        assert!(
            path.eq(&PathBuf::from("path/to/directory.txt")),
            "Path {} does not match path 'path/to/directory.txt'",
            path.to_str().unwrap_or("[COULD NOT DISPLAY]")
        );
    }
}
