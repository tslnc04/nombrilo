pub mod borrowed;
pub mod owned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

#[derive(Debug)]
pub struct TagTypeConversionError<T>(pub T);

impl<T> std::fmt::Display for TagTypeConversionError<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid tag type: {}", self.0)
    }
}

impl<T> std::error::Error for TagTypeConversionError<T> where T: std::fmt::Display + std::fmt::Debug {}

impl TryFrom<u8> for TagType {
    type Error = TagTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TagType::End),
            1 => Ok(TagType::Byte),
            2 => Ok(TagType::Short),
            3 => Ok(TagType::Int),
            4 => Ok(TagType::Long),
            5 => Ok(TagType::Float),
            6 => Ok(TagType::Double),
            7 => Ok(TagType::ByteArray),
            8 => Ok(TagType::String),
            9 => Ok(TagType::List),
            10 => Ok(TagType::Compound),
            11 => Ok(TagType::IntArray),
            12 => Ok(TagType::LongArray),
            _ => Err(TagTypeConversionError(value)),
        }
    }
}
