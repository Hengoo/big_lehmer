use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    ValidationDuplicateNumber,
    ValidationOutOfRange,
    DecodeError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationDuplicateNumber => {
                f.write_str("Input number sequence contains duplicate numbers")
            }
            Self::ValidationOutOfRange => {
                f.write_str("Input number sequence contains numbers larger than the sequence")
            }
            Self::DecodeError => f.write_str("Something failed during decode"),
        }
    }
}

impl std::error::Error for Error {}
