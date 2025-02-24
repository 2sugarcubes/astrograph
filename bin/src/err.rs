use serde_json::Error as ParseError;
use std::io::Error as IoError;

#[derive(Debug)]
pub(super) enum Error {
    Parse(ParseError),
    Write(IoError),
    Read(IoError),
    Multiple(Vec<Self>),
}

impl Error {
    pub(super) fn write_error(err: std::io::Error) -> Self {
        Self::Write(err)
    }

    pub(super) fn read_error(err: std::io::Error) -> Self {
        Self::Read(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Multiple(vec) => {
                assert!(vec.len() > 1);
                // HACK: flatten vec so that if it did contain a nested multiple type it would not
                // Issue URL: https://github.com/2sugarcubes/astrograph/issues/116
                // be nested e.g. "Multiple errors: Error 1, Error 2, Multiple errors: Error 3,
                // Error 4."
                write!(f, "Multiple Errors: {}", vec[0])?;

                for e in &vec[1..] {
                    write!(f, ", {e}")?;
                }
                write!(f, ".")
            }
            Self::Parse(e) => {
                write!(f, "Parsing Error: {e}.")
            }
            Self::Read(e) => {
                write!(f, "Read Error: {e}.")
            }
            Self::Write(e) => {
                write!(f, "Write Error: {e}.")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}
