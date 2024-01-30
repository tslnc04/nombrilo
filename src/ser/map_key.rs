use std::io::Write;

use serde::{
    ser::{self, Impossible},
    Serialize,
};

use super::formatter::Formatter;

pub(super) struct Serializer<'a, W, F> {
    writer: &'a mut W,
    formatter: &'a mut F,
}

impl<'a, W, F> Serializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    pub fn new(writer: &'a mut W, formatter: &'a mut F) -> Self {
        Self { writer, formatter }
    }
}

impl<'a, W, F> ser::Serializer for Serializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = super::Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, &v.to_string())?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_string(self.writer, v)?)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(super::Error::CompoundKey)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(super::Error::CompoundKey)
    }
}
