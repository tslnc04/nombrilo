use std::collections::HashMap;

use serde::{de::Visitor, Deserialize, Serialize};

use crate::unpack;

macro_rules! impl_array_deserialize {
    ($($array:ident)*) => {
        $(
            impl $array {
                pub fn new(inner: Vec<u8>) -> Self {
                    Self {
                        native_endian: cfg!(target_endian = "big"),
                        inner,
                    }
                }

                pub fn as_raw_slice(&self) -> &[u8] {
                    &self.inner
                }

                pub fn as_raw_slice_mut(&mut self) -> &mut [u8] {
                    &mut self.inner
                }

                pub fn is_empty(&self) -> bool {
                    self.inner.is_empty()
                }

                pub fn native_endian(&self) -> bool {
                    self.native_endian
                }

                pub fn big_endian(&self) -> bool {
                    #[cfg(target_endian = "big")]
                    {
                        self.native_endian
                    }
                    #[cfg(target_endian = "little")]
                    {
                        !self.native_endian
                    }
                }

                pub fn little_endian(&self) -> bool {
                    #[cfg(target_endian = "big")]
                    {
                        !self.native_endian
                    }
                    #[cfg(target_endian = "little")]
                    {
                        self.native_endian
                    }
                }
            }

            impl<'de> Deserialize<'de> for $array {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct ArrayVisitor;

                    impl<'de> Visitor<'de> for ArrayVisitor {
                        type Value = $array;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(formatter, "a {:?}", stringify!($array))
                        }

                        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok($array::new(v.to_vec()))
                        }

                        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok($array::new(v))
                        }
                    }

                    deserializer.deserialize_bytes(ArrayVisitor)
                }
            }
        )*
    };
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct ByteArray {
    native_endian: bool,
    inner: Vec<u8>,
}

impl ByteArray {
    pub fn as_slice(&self) -> &[i8] {
        unsafe { std::slice::from_raw_parts(self.inner.as_ptr() as *const i8, self.inner.len()) }
    }

    pub fn get(&self, index: usize) -> Option<i8> {
        self.inner.get(index).map(|x| *x as i8)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct IntArray {
    native_endian: bool,
    inner: Vec<u8>,
}

impl IntArray {
    pub fn as_slice(&mut self) -> &[i32] {
        if !self.native_endian {
            self.swap_endianness();
        }

        unsafe {
            std::slice::from_raw_parts(self.inner.as_ptr() as *const i32, self.inner.len() / 4)
        }
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        let index = index * 4;
        if index + 4 > self.inner.len() {
            return None;
        }

        if self.native_endian {
            Some(i32::from_ne_bytes([
                self.inner[index],
                self.inner[index + 1],
                self.inner[index + 2],
                self.inner[index + 3],
            ]))
        } else {
            Some(i32::from_be_bytes([
                self.inner[index],
                self.inner[index + 1],
                self.inner[index + 2],
                self.inner[index + 3],
            ]))
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len() / 4
    }

    fn swap_endianness(&mut self) {
        let swapped = unpack::swap_endianness_32bit(&self.inner);
        self.inner = swapped;
        self.native_endian = true;
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LongArray {
    native_endian: bool,
    inner: Vec<u8>,
}

impl LongArray {
    pub fn as_slice(&mut self) -> &[i64] {
        if !self.native_endian {
            self.swap_endianness();
        }

        unsafe {
            std::slice::from_raw_parts(self.inner.as_ptr() as *const i64, self.inner.len() / 8)
        }
    }

    pub fn get(&self, index: usize) -> Option<i64> {
        let index = index * 8;
        if index + 8 > self.inner.len() {
            return None;
        }

        if self.native_endian {
            Some(i64::from_ne_bytes([
                self.inner[index],
                self.inner[index + 1],
                self.inner[index + 2],
                self.inner[index + 3],
                self.inner[index + 4],
                self.inner[index + 5],
                self.inner[index + 6],
                self.inner[index + 7],
            ]))
        } else {
            Some(i64::from_be_bytes([
                self.inner[index],
                self.inner[index + 1],
                self.inner[index + 2],
                self.inner[index + 3],
                self.inner[index + 4],
                self.inner[index + 5],
                self.inner[index + 6],
                self.inner[index + 7],
            ]))
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len() / 8
    }

    fn swap_endianness(&mut self) {
        let swapped = unpack::swap_endianness_64bit(&self.inner);
        self.inner = swapped;
        self.native_endian = true;
    }
}

impl_array_deserialize! { ByteArray IntArray LongArray }

#[derive(Debug, Clone)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(ByteArray),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(IntArray),
    LongArray(LongArray),
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Tag::End => serializer.serialize_unit(),
            Tag::Byte(v) => serializer.serialize_i8(*v),
            Tag::Short(v) => serializer.serialize_i16(*v),
            Tag::Int(v) => serializer.serialize_i32(*v),
            Tag::Long(v) => serializer.serialize_i64(*v),
            Tag::Float(v) => serializer.serialize_f32(*v),
            Tag::Double(v) => serializer.serialize_f64(*v),
            Tag::ByteArray(v) => v.serialize(serializer),
            Tag::String(v) => serializer.serialize_str(v),
            Tag::List(v) => v.serialize(serializer),
            Tag::Compound(v) => v.serialize(serializer),
            Tag::IntArray(v) => v.serialize(serializer),
            Tag::LongArray(v) => v.serialize(serializer),
        }
    }
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
