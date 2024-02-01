use serde::de;

use crate::nbt::TagType;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(std::io::Error),
    InvalidBooleanValue(i8),
    NegativeUnsigned,
    NegativeLength,
    InvalidTopLevel(TagType),
    InvalidMUTF8,
    UnexpectedEndTag,
    InvalidTagForSeq(TagType),
    InvalidTagType(u8),
    InvalidTagForBytes(TagType),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Io(e) => write!(f, "{}", e),
            Error::InvalidBooleanValue(byte) => {
                write!(f, "invalid boolean value: {}", byte)
            }
            Error::NegativeUnsigned => write!(f, "negative unsigned integer"),
            Error::NegativeLength => write!(f, "negative length"),
            Error::InvalidTopLevel(tag_type) => {
                write!(f, "invalid top level tag type: {:?}", tag_type)
            }
            Error::InvalidMUTF8 => write!(f, "invalid MUTF-8 string"),
            Error::UnexpectedEndTag => write!(f, "unexpected end tag"),
            Error::InvalidTagForSeq(tag_type) => {
                write!(f, "invalid tag type for sequence: {:?}", tag_type)
            }
            Error::InvalidTagType(byte) => write!(f, "invalid tag type: {}", byte),
            Error::InvalidTagForBytes(tag_type) => {
                write!(f, "invalid tag type for byte array: {:?}", tag_type)
            }
        }
    }
}

impl std::error::Error for Error {}
