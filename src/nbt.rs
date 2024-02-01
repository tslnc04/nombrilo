use core::slice;
use std::{borrow::Cow, collections::HashMap};

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

macro_rules! impl_array_deserialize {
    ($($array:ident)*) => {
        $(
            impl<'a> $array<'a> {
                pub fn new(inner: Cow<'a, [u8]>) -> Self {
                    Self {
                        endian_swapped: cfg!(target_endian = "big"),
                        inner,
                    }
                }

                pub fn as_raw_slice(&self) -> &[u8] {
                    &self.inner
                }

                pub fn as_raw_slice_mut(&mut self) -> &mut [u8] {
                    self.inner.to_mut()
                }
            }

            impl<'de, 'a> Deserialize<'de> for $array<'a>
            where
                'de: 'a,
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct ArrayVisitor;

                    impl<'de> Visitor<'de> for ArrayVisitor {
                        type Value = $array<'de>;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(formatter, "a {:?}", stringify!($array))
                        }

                        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok($array::new(Cow::Owned(v.to_vec())))
                        }

                        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok($array::new(Cow::Borrowed(v)))
                        }

                        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok($array::new(Cow::Owned(v)))
                        }
                    }

                    deserializer.deserialize_bytes(ArrayVisitor)
                }
            }
        )*
    };
}

#[derive(Debug, Clone, Serialize)]
pub struct ByteArray<'a> {
    endian_swapped: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> ByteArray<'a> {
    pub fn as_slice(&self) -> &'a [i8] {
        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i8, self.inner.len()) }
    }

    pub fn to_vec(&self) -> Vec<i8> {
        self.as_slice().to_vec()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IntArray<'a> {
    endian_swapped: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> IntArray<'a> {
    pub fn as_slice(&mut self) -> &'a [i32] {
        if !self.endian_swapped {
            self.swap_endianness();
        }

        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i32, self.inner.len() / 4) }
    }

    pub fn to_vec(&mut self) -> Vec<i32> {
        self.as_slice().to_vec()
    }

    fn swap_endianness(&mut self) {
        for i in 0..self.inner.len() / 4 {
            let x = u32::from_be_bytes([
                self.inner[i * 4],
                self.inner[i * 4 + 1],
                self.inner[i * 4 + 2],
                self.inner[i * 4 + 3],
            ]);
            self.inner.to_mut()[i * 4..(i + 1) * 4].copy_from_slice(&x.to_le_bytes());
        }
        self.endian_swapped = true;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LongArray<'a> {
    endian_swapped: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> LongArray<'a> {
    pub fn as_slice(&mut self) -> &'a [i64] {
        if !self.endian_swapped {
            self.swap_endianness();
        }

        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i64, self.inner.len() / 8) }
    }

    pub fn to_vec(&mut self) -> Vec<i64> {
        self.as_slice().to_vec()
    }

    fn swap_endianness(&mut self) {
        for i in 0..self.inner.len() / 8 {
            let x = u64::from_be_bytes([
                self.inner[i * 8],
                self.inner[i * 8 + 1],
                self.inner[i * 8 + 2],
                self.inner[i * 8 + 3],
                self.inner[i * 8 + 4],
                self.inner[i * 8 + 5],
                self.inner[i * 8 + 6],
                self.inner[i * 8 + 7],
            ]);
            self.inner.to_mut()[i * 8..(i + 1) * 8].copy_from_slice(&x.to_le_bytes());
        }
        self.endian_swapped = true;
    }
}

impl_array_deserialize! { ByteArray IntArray LongArray }
