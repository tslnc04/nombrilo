use std::{borrow::Cow, slice};

use serde::{de::Visitor, Deserialize, Serialize};

use crate::unpack;

macro_rules! impl_array_deserialize {
    ($($array:ident)*) => {
        $(
            impl<'a> $array<'a> {
                pub fn new(inner: Cow<'a, [u8]>) -> Self {
                    Self {
                        native_endian: cfg!(target_endian = "big"),
                        inner,
                    }
                }

                pub fn as_raw_slice(&self) -> &[u8] {
                    &self.inner
                }

                pub fn as_raw_slice_mut(&mut self) -> &mut [u8] {
                    self.inner.to_mut()
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
    native_endian: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> ByteArray<'a> {
    pub fn as_slice(&self) -> &'a [i8] {
        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i8, self.inner.len()) }
    }

    pub fn to_vec(&self) -> Vec<i8> {
        self.as_slice().to_vec()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IntArray<'a> {
    native_endian: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> IntArray<'a> {
    pub fn as_slice(&mut self) -> &'a [i32] {
        if !self.native_endian {
            self.swap_endianness();
        }

        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i32, self.inner.len() / 4) }
    }

    pub fn to_vec(&mut self) -> Vec<i32> {
        self.as_slice().to_vec()
    }

    pub fn len(&self) -> usize {
        self.inner.len() / 4
    }

    fn swap_endianness(&mut self) {
        let swapped = unpack::swap_endianness_32bit(self.inner.as_ref());
        self.inner = Cow::Owned(swapped);
        self.native_endian = true;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LongArray<'a> {
    native_endian: bool,
    inner: Cow<'a, [u8]>,
}

impl<'a> LongArray<'a> {
    pub fn as_slice(&mut self) -> &'a [i64] {
        if !self.native_endian {
            self.swap_endianness();
        }

        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const i64, self.inner.len() / 8) }
    }

    pub fn to_vec(&mut self) -> Vec<i64> {
        self.as_slice().to_vec()
    }

    pub fn len(&self) -> usize {
        self.inner.len() / 8
    }

    fn swap_endianness(&mut self) {
        let swapped = unpack::swap_endianness_64bit(self.inner.as_ref());
        self.inner = Cow::Owned(swapped);
        self.native_endian = true;
    }
}

impl_array_deserialize! { ByteArray IntArray LongArray }
