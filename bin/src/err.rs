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

    fn into_vec(self) -> Vec<Self> {
        match self {
            Self::Parse(_) | Self::Read(_) | Self::Write(_) => vec![self],
            Self::Multiple(vec) => {
                // Recursively search for "multiple" type errors to flatten them into one level
                vec.into_iter().flat_map(Self::into_vec).collect()
            }
        }
    }

    pub fn flatten(self) -> Self {
        match self {
            Self::Parse(_) | Self::Read(_) | Self::Write(_) => self,
            Self::Multiple(_) => {
                // Map any nested multiple errors into one level
                Self::Multiple(self.into_vec())
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Multiple(vec) => {
                assert!(vec.len() > 1);

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
