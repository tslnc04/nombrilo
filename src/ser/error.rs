use std::fmt::Display;

use serde::ser;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(std::io::Error),
    UnsignedTooBig,
    StringTooBig,
    SequenceTooBig,
    UnknownLength,
    CompoundKey,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Io(e) => write!(f, "{}", e),
            Error::UnsignedTooBig => write!(
                f,
                "UnsignedTooBig: unsigned integers must fit into equivalent signed type"
            ),
            Error::StringTooBig => write!(f, "StringTooBig: strings must be at most 65535 bytes"),
            Error::SequenceTooBig => write!(
                f,
                "SequenceTooBig: sequences must be at most 2^31 - 1 elements"
            ),
            Error::UnknownLength => {
                write!(f, "UnknownLength: all sequences must have known length")
            }
            Error::CompoundKey => write!(
                f,
                "CompoundKey: map keys must be non-None and non-unit scalar types"
            ),
        }
    }
}

impl std::error::Error for Error {}
