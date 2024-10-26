use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use coordinates::prelude::Spherical;
use dyn_clone::DynClone;

use crate::{body::ArcBody, Float};

pub mod svg;

pub trait Output: DynClone + Debug {
    fn write_observations_to_file(
        &self,
        observations: &[(ArcBody, Spherical<Float>)],
        path: &Path,
    ) -> Result<(), std::io::Error>;
}
dyn_clone::clone_trait_object!(Output);

fn set_extension(path: &PathBuf, extension: &str) -> PathBuf {
    let mut path = path.clone();
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
