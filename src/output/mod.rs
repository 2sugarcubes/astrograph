use std::path::PathBuf;

use coordinates::prelude::Spherical;

use crate::{body::ArcBody, Float};

pub mod svg;

pub trait Output {
    type OutType;
    fn consume_observation(&self, observations: &Vec<(ArcBody, Spherical<Float>)>)
        -> Self::OutType;

    // TODO create a more readable result type
    fn write_to_file(&self, contents: Self::OutType, path: &PathBuf) -> Result<(), std::io::Error>;
}

fn set_extension(path: &PathBuf, extension: &str) -> PathBuf {
    let mut path = path.clone();
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
