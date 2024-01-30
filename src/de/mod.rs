use serde::{
    de::{self, Deserialize, DeserializeOwned, DeserializeSeed, Visitor},
    forward_to_deserialize_any,
};

use crate::nbt::TagType;

use self::{error::Error, read::Reference};

mod error;
mod read;

struct Deserializer<R> {
    reader: R,
    scratch: Vec<u8>,
}

impl<'de, R> Deserializer<R>
where
    R: read::Read<'de>,
{
    fn new(reader: R) -> Self {
        Deserializer {
            reader,
            scratch: Vec::new(),
        }
    }
}

pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    let mut de = Deserializer::new(read::Reader::new(reader));
    let value = Deserialize::deserialize(&mut de)?;
    Ok(value)
}

pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut de = Deserializer::new(read::Slice::new(slice));
    let value = Deserialize::deserialize(&mut de)?;
    Ok(value)
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: read::Read<'de>,
{
    type Error = Error;

    forward_to_deserialize_any! { bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any }

    // Assume that this is the top level since everything else is deserialized
    // with UnnamedDeserializer
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let tag_type = self.reader.read_tag_type()?;
        self.reader.ignore_string()?;

        match tag_type {
            TagType::List => {
                let element_type = self.reader.read_tag_type()?;
                let len = self.reader.read_int()?;
                visitor.visit_seq(SeqAccess::new(
                    self,
                    element_type,
                    usize::try_from(len).map_err(|_| Error::NegativeLength)?,
                ))
            }
            TagType::Compound => visitor.visit_map(MapAccess::new(self)),
            _ => Err(Error::InvalidTopLevel(tag_type)),
        }
    }
}

struct UnnamedDeserializer<'a, R> {
    de: &'a mut Deserializer<R>,
    tag_type: TagType,
}

impl<'de, 'a, R> UnnamedDeserializer<'a, R>
where
    R: read::Read<'de>,
{
    fn new(de: &'a mut Deserializer<R>, tag_type: TagType) -> Self {
        UnnamedDeserializer { de, tag_type }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for UnnamedDeserializer<'a, R>
where
    R: read::Read<'de>,
{
    type Error = Error;

    forward_to_deserialize_any! { i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf unit unit_struct newtype_struct tuple tuple_struct struct enum identifier ignored_any }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.tag_type {
            TagType::Byte => visitor.visit_i8(self.de.reader.read_byte()?),
            TagType::Short => visitor.visit_i16(self.de.reader.read_short()?),
            TagType::Int => visitor.visit_i32(self.de.reader.read_int()?),
            TagType::Long => visitor.visit_i64(self.de.reader.read_long()?),
            TagType::Float => visitor.visit_f32(self.de.reader.read_float()?),
            TagType::Double => visitor.visit_f64(self.de.reader.read_double()?),
            TagType::ByteArray | TagType::IntArray | TagType::LongArray | TagType::List => {
                self.deserialize_seq(visitor)
            }
            TagType::String => match self.de.reader.read_string(&mut self.de.scratch)? {
                Reference::Copied(s) => visitor.visit_str(s),
                Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
            },
            TagType::Compound => self.deserialize_map(visitor),
            TagType::End => Err(Error::UnexpectedEndTag),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let byte = self.de.reader.read_byte()?;
        match byte {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            n => Err(Error::InvalidBooleanValue(n)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let element_type = match self.tag_type {
            TagType::ByteArray => TagType::Byte,
            TagType::IntArray => TagType::Int,
            TagType::LongArray => TagType::Long,
            TagType::List => self.de.reader.read_tag_type()?,
            _ => Err(Error::InvalidTagForSeq(self.tag_type))?,
        };
        let len = self.de.reader.read_int()?;
        visitor.visit_seq(SeqAccess::new(
            self.de,
            element_type,
            usize::try_from(len).map_err(|_| Error::NegativeLength)?,
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapAccess::new(self.de))
    }
}

struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    value_type: TagType,
}

impl<'de, 'a, R> MapAccess<'a, R>
where
    R: read::Read<'de>,
{
    fn new(de: &'a mut Deserializer<R>) -> Self {
        MapAccess {
            de,
            value_type: TagType::End,
        }
    }
}

impl<'de, 'a, R> de::MapAccess<'de> for MapAccess<'a, R>
where
    R: read::Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.read_tag_type()? {
            TagType::End => return Ok(None),
            tt => self.value_type = tt,
        }

        seed.deserialize(UnnamedDeserializer::new(self.de, TagType::String))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(UnnamedDeserializer::new(self.de, self.value_type))
    }
}

struct SeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    element_type: TagType,
    remaining: usize,
}

impl<'de, 'a, R> SeqAccess<'a, R>
where
    R: read::Read<'de>,
{
    fn new(de: &'a mut Deserializer<R>, element_type: TagType, remaining: usize) -> Self {
        SeqAccess {
            de,
            element_type,
            remaining,
        }
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for SeqAccess<'a, R>
where
    R: read::Read<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(UnnamedDeserializer::new(self.de, self.element_type))
            .map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ExampleNBT {
        pub name: String,
        pub age: u16,
        pub inventory: Vec<Item>,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Item {
        pub name: String,
        pub count: i32,
    }

    fn generate_example_output() -> ExampleNBT {
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

    fn generate_example_vec() -> Vec<u8> {
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

    fn generate_example_reader() -> impl std::io::Read {
        Cursor::new(generate_example_vec())
    }

    #[test]
    fn test_from_slice() {
        let example_nbt: ExampleNBT = from_slice(&generate_example_vec()).unwrap();
        assert_eq!(example_nbt, generate_example_output());
    }

    #[test]
    fn test_from_reader() {
        let example_nbt: ExampleNBT = from_reader(generate_example_reader()).unwrap();
        assert_eq!(example_nbt, generate_example_output());
    }
}
