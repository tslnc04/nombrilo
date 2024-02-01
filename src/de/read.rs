use std::{borrow::Cow, io};

use crate::nbt::{TagType, TagTypeConversionError};

use super::Error;

pub(super) enum Reference<'b, 'c, T>
where
    T: ?Sized,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

pub(super) trait Read<'de> {
    fn read_byte(&mut self) -> Result<i8, Error>;
    fn read_short(&mut self) -> Result<i16, Error>;
    fn read_int(&mut self) -> Result<i32, Error>;
    fn read_long(&mut self) -> Result<i64, Error>;
    fn read_float(&mut self) -> Result<f32, Error>;
    fn read_double(&mut self) -> Result<f64, Error>;

    fn read_bytes<'s>(
        &'s mut self,
        len_multiplier: usize,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>, Error>;

    fn read_string<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, str>, Error>;
    fn ignore_string(&mut self) -> Result<(), Error>;

    fn read_tag_type(&mut self) -> Result<TagType, Error> {
        // TODO(tslnc04): figure out a better way to convert into TagType
        (self.read_byte()? as u8)
            .try_into()
            .map_err(|err: TagTypeConversionError<u8>| Error::InvalidTagType(err.0))
    }
}

pub(super) struct Reader<R> {
    reader: R,
}

impl<R> Reader<R>
where
    R: io::Read,
{
    pub(super) fn new(reader: R) -> Self {
        Reader { reader }
    }
}

impl<'a, R> Read<'a> for Reader<R>
where
    R: io::Read,
{
    fn read_byte(&mut self) -> Result<i8, Error> {
        let mut buf = [0];
        self.reader.read_exact(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    fn read_short(&mut self) -> Result<i16, Error> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_int(&mut self) -> Result<i32, Error> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_long(&mut self) -> Result<i64, Error> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    fn read_float(&mut self) -> Result<f32, Error> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }

    fn read_double(&mut self) -> Result<f64, Error> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn read_bytes<'s>(
        &'s mut self,
        len_multiplier: usize,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, [u8]>, Error> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        let len = usize::try_from(i32::from_be_bytes(buf)).map_err(|_| Error::NegativeLength)?;
        scratch.resize(len * len_multiplier, 0);
        self.reader.read_exact(scratch.as_mut_slice())?;
        Ok(Reference::Copied(scratch.as_slice()))
    }

    fn read_string<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, str>, Error> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        let len = u16::from_be_bytes(buf);
        scratch.resize(len as usize, 0);
        self.reader.read_exact(scratch.as_mut_slice())?;
        let converted =
            cesu8::from_java_cesu8(scratch.as_slice()).map_err(|_| Error::InvalidMUTF8)?;
        match converted {
            Cow::Borrowed(_) => Ok(Reference::Copied(unsafe {
                std::str::from_utf8_unchecked(scratch.as_slice())
            })),
            Cow::Owned(s) => {
                *scratch = s.into_bytes();
                Ok(Reference::Copied(unsafe {
                    std::str::from_utf8_unchecked(scratch.as_slice())
                }))
            }
        }
    }

    fn ignore_string(&mut self) -> Result<(), Error> {
        let len = self.read_short()?;
        let mut buf = vec![0; len as usize];
        self.reader.read_exact(&mut buf)?;
        Ok(())
    }
}

pub(super) struct Slice<'a> {
    slice: &'a [u8],
}

impl<'a> Slice<'a> {
    pub(super) fn new(slice: &'a [u8]) -> Self {
        Slice { slice }
    }
}

impl<'a> Read<'a> for Slice<'a> {
    fn read_byte(&mut self) -> Result<i8, Error> {
        let (byte, rest) = self.slice.split_at(1);
        self.slice = rest;
        Ok(i8::from_be_bytes([byte[0]]))
    }

    fn read_short(&mut self) -> Result<i16, Error> {
        let (short, rest) = self.slice.split_at(2);
        self.slice = rest;
        Ok(i16::from_be_bytes([short[0], short[1]]))
    }

    fn read_int(&mut self) -> Result<i32, Error> {
        let (int, rest) = self.slice.split_at(4);
        self.slice = rest;
        Ok(i32::from_be_bytes([int[0], int[1], int[2], int[3]]))
    }

    fn read_long(&mut self) -> Result<i64, Error> {
        let (long, rest) = self.slice.split_at(8);
        self.slice = rest;
        Ok(i64::from_be_bytes([
            long[0], long[1], long[2], long[3], long[4], long[5], long[6], long[7],
        ]))
    }

    fn read_float(&mut self) -> Result<f32, Error> {
        let (float, rest) = self.slice.split_at(4);
        self.slice = rest;
        Ok(f32::from_be_bytes([float[0], float[1], float[2], float[3]]))
    }

    fn read_double(&mut self) -> Result<f64, Error> {
        let (double, rest) = self.slice.split_at(8);
        self.slice = rest;
        Ok(f64::from_be_bytes([
            double[0], double[1], double[2], double[3], double[4], double[5], double[6], double[7],
        ]))
    }

    fn read_bytes<'s>(
        &'s mut self,
        len_multiplier: usize,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, [u8]>, Error> {
        let (len_bytes, rest) = self.slice.split_at(4);
        self.slice = rest;
        let len = i32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]);
        let (bytes, rest) = self
            .slice
            .split_at(usize::try_from(len).map_err(|_| Error::NegativeLength)? * len_multiplier);
        self.slice = rest;
        Ok(Reference::Borrowed(bytes))
    }

    fn read_string<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, str>, Error> {
        let (len_bytes, rest) = self.slice.split_at(2);
        self.slice = rest;
        let len = u16::from_be_bytes([len_bytes[0], len_bytes[1]]);
        let (string, rest) = self.slice.split_at(len as usize);
        self.slice = rest;
        let converted = cesu8::from_java_cesu8(string).map_err(|_| Error::InvalidMUTF8)?;
        match converted {
            Cow::Borrowed(s) => Ok(Reference::Borrowed(s)),
            Cow::Owned(s) => {
                *scratch = s.into_bytes();
                Ok(Reference::Copied(unsafe {
                    std::str::from_utf8_unchecked(scratch.as_slice())
                }))
            }
        }
    }

    fn ignore_string(&mut self) -> Result<(), Error> {
        let len = self.read_short()?;
        let (_string, rest) = self.slice.split_at(len as usize);
        self.slice = rest;
        Ok(())
    }
}
