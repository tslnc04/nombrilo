use std::io::{Cursor, Write};

use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::nbt::TagType;

use self::{
    error::Error,
    formatter::{BinaryFormatter, Formatter, StringifiedFormatter},
    tag_type::to_tag_type,
};

mod error;
mod formatter;
mod map_key;
mod tag_type;

struct Serializer<W, F = BinaryFormatter> {
    writer: W,
    formatter: F,
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)
}

pub fn to_snbt_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = Serializer::with_formatter(writer, StringifiedFormatter::new());
    value.serialize(&mut serializer)
}

impl<W> Serializer<W>
where
    W: Write,
{
    fn new(writer: W) -> Self {
        Serializer {
            writer,
            formatter: BinaryFormatter::new(),
        }
    }
}

impl<W, F> Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer { writer, formatter }
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, W, F>;
    type SerializeTuple = SeqSerializer<'a, W, F>;
    type SerializeTupleStruct = SeqSerializer<'a, W, F>;
    type SerializeTupleVariant = SeqSerializer<'a, W, F>;
    type SerializeMap = MapSerializer<'a, W, F>;
    type SerializeStruct = MapSerializer<'a, W, F>;
    type SerializeStructVariant = MapSerializer<'a, W, F>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_bool(&mut self.writer, v)?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_byte(&mut self.writer, v)?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_short(&mut self.writer, v)?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_int(&mut self.writer, v)?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_long(&mut self.writer, v)?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_byte(
            &mut self.writer,
            i8::try_from(v).map_err(|_| Error::UnsignedTooBig)?,
        )?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_short(
            &mut self.writer,
            i16::try_from(v).map_err(|_| Error::UnsignedTooBig)?,
        )?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_int(
            &mut self.writer,
            i32::try_from(v).map_err(|_| Error::UnsignedTooBig)?,
        )?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_long(
            &mut self.writer,
            i64::try_from(v).map_err(|_| Error::UnsignedTooBig)?,
        )?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_float(&mut self.writer, v)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_double(&mut self.writer, v)?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if v.len() > u16::MAX as usize {
            return Err(Error::StringTooBig);
        }

        Ok(self.formatter.write_string(&mut self.writer, v)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(self.formatter.write_byte_array(&mut self.writer, v)?)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
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
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            if len > i32::MAX as usize {
                Err(Error::SequenceTooBig)
            } else {
                Ok(SeqSerializer::new(self, len as i32))
            }
        } else {
            Err(Error::UnknownLength)
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.formatter.start_compound(&mut self.writer)?;
        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

struct SeqSerializer<'a, W, F> {
    serializer: &'a mut Serializer<W, F>,
    len: i32,
    first: bool,
}

impl<'a, W, F> SeqSerializer<'a, W, F> {
    fn new(serializer: &'a mut Serializer<W, F>, len: i32) -> Self {
        SeqSerializer {
            serializer,
            len,
            first: true,
        }
    }
}

impl<'a, W, F> SerializeSeq for SeqSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let first = self.first;
        if self.first {
            self.first = false;
            match to_tag_type(value)? {
                TagType::Byte => self
                    .serializer
                    .formatter
                    .start_byte_array(&mut self.serializer.writer, self.len)?,
                TagType::Int => self
                    .serializer
                    .formatter
                    .start_int_array(&mut self.serializer.writer, self.len)?,
                TagType::Long => self
                    .serializer
                    .formatter
                    .start_long_array(&mut self.serializer.writer, self.len)?,
                element_type => self.serializer.formatter.start_list(
                    &mut self.serializer.writer,
                    self.len,
                    element_type,
                )?,
            }
        }

        self.serializer
            .formatter
            .start_element(&mut self.serializer.writer, first)?;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self
            .serializer
            .formatter
            .end_sequence(&mut self.serializer.writer)?)
    }
}

impl<'a, W, F> SerializeTuple for SeqSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a, W, F> SerializeTupleStruct for SeqSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a, W, F> SerializeTupleVariant for SeqSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

struct MapSerializer<'a, W, F> {
    serializer: &'a mut Serializer<W, F>,
    key: Cursor<Vec<u8>>,
}

impl<'a, W, F> MapSerializer<'a, W, F> {
    fn new(serializer: &'a mut Serializer<W, F>) -> Self {
        MapSerializer {
            serializer,
            key: Cursor::new(Vec::new()),
        }
    }
}

impl<'a, W, F> SerializeMap for MapSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.key.get_mut().clear();
        self.key.set_position(0);
        key.serialize(map_key::Serializer::new(
            &mut self.key,
            &mut self.serializer.formatter,
        ))?;
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value_type = to_tag_type(value)?;
        self.serializer.formatter.start_entry(
            &mut self.serializer.writer,
            self.key.get_ref(),
            value_type,
        )?;
        value.serialize(&mut *self.serializer)?;
        self.serializer
            .formatter
            .end_entry(&mut self.serializer.writer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self
            .serializer
            .formatter
            .end_compound(&mut self.serializer.writer)?)
    }
}

impl<'a, W, F> SerializeStruct for MapSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl<'a, W, F> SerializeStructVariant for MapSerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Serialize;

    #[derive(Serialize)]
    struct ExampleNBT {
        pub name: String,
        pub age: u16,
        pub inventory: Vec<Item>,
    }

    #[derive(Serialize)]
    struct Item {
        pub name: String,
        pub count: i32,
    }

    fn generate_example() -> ExampleNBT {
        let items = vec![
            Item {
                name: "test".to_string(),
                count: 1,
            },
            Item {
                name: "test2".to_string(),
                count: 2,
            },
        ];
        ExampleNBT {
            name: "test nbt".to_string(),
            age: 40,
            inventory: items,
        }
    }

    fn generate_example_nbt() -> Vec<u8> {
        vec![
            0x0a, 0x00, 0x00, // compound, empty name
            0x08, // string
            0x00, 0x04, 0x6e, 0x61, 0x6d, 0x65, // name "name"
            0x00, 0x08, 0x74, 0x65, 0x73, 0x74, 0x20, 0x6e, 0x62, 0x74, // value "test nbt"
            0x02, // short
            0x00, 0x03, 0x61, 0x67, 0x65, // name "age"
            0x00, 0x28, // value 40
            0x09, 0x00, 0x09, 0x69, 0x6e, 0x76, 0x65, 0x6e, 0x74, 0x6f, 0x72,
            0x79, // name "inventory"
            0x0a, 0x00, 0x00, 0x00, 0x02, // list of type compound, len 2
            0x08, // string
            0x00, 0x04, 0x6e, 0x61, 0x6d, 0x65, // name "name"
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // value "test"
            0x03, // int
            0x00, 0x05, 0x63, 0x6f, 0x75, 0x6e, 0x74, // name "count"
            0x00, 0x00, 0x00, 0x01, // value 1
            0x00, // end tag
            0x08, // string
            0x00, 0x04, 0x6e, 0x61, 0x6d, 0x65, // name "name"
            0x00, 0x05, 0x74, 0x65, 0x73, 0x74, 0x32, // value "test2"
            0x03, // int
            0x00, 0x05, 0x63, 0x6f, 0x75, 0x6e, 0x74, // name "count"
            0x00, 0x00, 0x00, 0x02, // value 1
            0x00, // end tag
            0x00, // end tag
        ]
    }

    fn generate_example_snbt() -> String {
        "{\"name\":\"test nbt\",\"age\":40s,\"inventory\":[{\"name\":\"test\",\"count\":1},{\"name\":\"test2\",\"count\":2}]}".to_string()
    }

    #[test]
    fn test_nbt() {
        let example_nbt = generate_example();
        let mut buffer = Cursor::new(Vec::<u8>::new());
        to_writer(&mut buffer, &example_nbt).unwrap();
        assert_eq!(buffer.get_ref(), &generate_example_nbt());
    }

    #[test]
    fn test_snbt() {
        let example_nbt = generate_example();
        let mut buffer = Cursor::new(Vec::<u8>::new());
        to_snbt_writer(&mut buffer, &example_nbt).unwrap();
        assert_eq!(
            String::from_utf8_lossy(buffer.get_ref()),
            generate_example_snbt(),
        );
    }
}
