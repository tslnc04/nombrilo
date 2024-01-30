use std::collections::HashMap;

use serde::{de::Visitor, Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TagVisitor;

        impl<'de> Visitor<'de> for TagVisitor {
            type Value = Tag;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid NBT tag")
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(Tag::Byte(v as i8))
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Byte(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Short(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Int(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Long(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Float(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::Double(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::String(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::String(v))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut tags = Vec::new();
                if let Some(len) = seq.size_hint() {
                    tags.reserve_exact(len);
                }

                while let Some(tag) = seq.next_element::<Tag>()? {
                    tags.push(tag);
                }

                // pretend that every sequence is a list
                Ok(Tag::List(tags))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut tags = HashMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    tags.insert(key, value);
                }

                Ok(Tag::Compound(tags))
            }
        }

        deserializer.deserialize_any(TagVisitor)
    }
}
