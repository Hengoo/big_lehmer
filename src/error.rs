use std::fmt::{self};
use std::fmt::{Display, Formatter};

use crate::Lehmer;

#[derive(Debug)]
pub enum Error {
    ValidationDuplicateNumber,
    ValidationOutOfRange,
    SequenceToLong {
        element_count: usize,
    },
    OutVectorSize {
        byte_size: usize,
        element_count: u32,
    },
    Decode,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationDuplicateNumber => {
                f.write_str("Input number sequence contains duplicate numbers")
            }
            Self::ValidationOutOfRange => f.write_str(
                "Input number sequence contains numbers larger than the sequence length",
            ),
            Self::SequenceToLong { element_count } => f.write_fmt(format_args!(
                "Input sequence contains {element_count} elements, but we only support up to 2^32"
            )),
            Self::Decode => f.write_str("Something failed during decode. Usually happens from invalid input."),
            Self::OutVectorSize {
                byte_size,
                element_count,
            } => f.write_fmt(format_args!(
                // Breaking up the string because long string causes bugs with the code auto formatting :(
                "{byte_size} {element_count} byte, but storing its {} elements requires {} byte. {}",
                "The byte output vector used in encode has",
                Lehmer::get_encode_size(*element_count),
                "Make sure to correctly use \"Lehmer::get_encode_size()\""
            )),
        }
    }
}

impl std::error::Error for Error {}
