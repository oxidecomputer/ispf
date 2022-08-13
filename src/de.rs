// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use std::str::from_utf8;

use crate::LittleEndian;
use serde::de::{self, DeserializeSeed, SeqAccess, Visitor};
use serde::Deserialize;

pub trait NumDe {
    fn deserialize_u16(v: [u8; 2]) -> u16;
    fn deserialize_u32(v: [u8; 4]) -> u32;
    fn deserialize_u64(v: [u8; 8]) -> u64;
}

impl NumDe for LittleEndian {
    fn deserialize_u16(v: [u8; 2]) -> u16 {
        u16::from_le_bytes(v)
    }
    fn deserialize_u32(v: [u8; 4]) -> u32 {
        u32::from_le_bytes(v)
    }
    fn deserialize_u64(v: [u8; 8]) -> u64 {
        u64::from_le_bytes(v)
    }
}

trait ReadSize {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize>;
}

impl ReadSize for u8 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        match bytes.first() {
            Some(x) => Ok(*x as usize),
            None => Err(Error::ExpectedInteger),
        }
    }
}

impl ReadSize for u16 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u16(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?,
        ) as usize)
    }
}

impl ReadSize for u32 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u32(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?,
        ) as usize)
    }
}

impl ReadSize for u64 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u64(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?,
        ) as usize)
    }
}

use crate::error::{Error, Result};

pub struct Deserializer<'de, Endian: NumDe> {
    input: &'de [u8],
    endian: PhantomData<Endian>,
}

impl<'de, Endian: NumDe> Deserializer<'de, Endian> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input,
            endian: PhantomData::<Endian> {},
        }
    }

    fn read_tlv_string<T: ReadSize>(&mut self) -> Result<&'de str> {
        use std::mem::size_of;

        let n = size_of::<T>();

        let len = T::read_size::<Endian>(&self.input[..n])?;
        let s = from_utf8(&self.input[n..n + len]).map_err(|_| Error::Eof)?;

        self.input = &self.input[n + len..];
        Ok(s)
    }
}

pub fn from_bytes_le<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    from_bytes::<'a, LittleEndian, T>(b)
}

pub fn from_bytes<'a, Endian, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
    Endian: NumDe,
{
    let mut deserializer = Deserializer::<'a, Endian>::from_bytes(b);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub struct TlvStringVisitor;
impl<'de> Visitor<'de> for TlvStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string prifixed by a length")
    }

    fn visit_borrowed_str<E>(
        self,
        value: &'de str,
    ) -> core::result::Result<Self::Value, E> {
        Ok(value.to_string())
    }
}

pub struct TlvVecVisitor<'de, T: serde::Deserialize<'de>> {
    phantom: PhantomData<T>,
    of_the_opera: PhantomData<&'de ()>,
}

impl<'de, T: serde::Deserialize<'de>> TlvVecVisitor<'de, T> {
    pub fn new() -> Self {
        TlvVecVisitor {
            phantom: PhantomData::<T> {},
            of_the_opera: PhantomData::<&'de ()> {},
        }
    }
}

impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for TlvVecVisitor<'de, T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array prifixed by a length")
    }

    fn visit_seq<A>(
        self,
        mut seq: A,
    ) -> core::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut value = Vec::new();
        while let Some(x) = seq.next_element()? {
            value.push(x)
        }
        Ok(value)
    }
}

struct PackedArray<'a, 'de: 'a, Endian: NumDe> {
    de: &'a mut Deserializer<'de, Endian>,
    count: usize,
}

impl<'de, 'a, Endian: NumDe> PackedArray<'a, 'de, Endian> {
    fn new(de: &'a mut Deserializer<'de, Endian>, count: usize) -> Self {
        PackedArray { de, count }
    }
}

impl<'de, 'a, Endian: NumDe> SeqAccess<'de> for PackedArray<'a, 'de, Endian> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        self.count -= 1;
        if self.count == 0 {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct PackedArrayByteSized<'a, 'de: 'a, Endian: NumDe> {
    de: &'a mut Deserializer<'de, Endian>,
    bytes: usize,
}

impl<'de, 'a, Endian: NumDe> PackedArrayByteSized<'a, 'de, Endian> {
    fn new(de: &'a mut Deserializer<'de, Endian>, bytes: usize) -> Self {
        PackedArrayByteSized { de, bytes }
    }
}

impl<'de, 'a, Endian: NumDe> SeqAccess<'de>
    for PackedArrayByteSized<'a, 'de, Endian>
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.bytes == 0 {
            return Ok(None);
        }
        let before = self.de.input.len();
        let res = seed.deserialize(&mut *self.de).map(Some);
        let after = self.de.input.len();
        self.bytes -= before - after;
        res
    }
}

impl<'de, 'a, Endian: NumDe> de::Deserializer<'de>
    for &'a mut Deserializer<'de, Endian>
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.input[0];
        self.input = &self.input[1..];
        visitor.visit_u8(byte)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.input[..2].try_into().map_err(|_| Error::Eof)?;
        self.input = &self.input[2..];
        visitor.visit_u16(Endian::deserialize_u16(bytes))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.input[..4].try_into().map_err(|_| Error::Eof)?;
        self.input = &self.input[4..];
        visitor.visit_u32(Endian::deserialize_u32(bytes))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.input[..8].try_into().map_err(|_| Error::Eof)?;
        self.input = &self.input[8..];
        visitor.visit_u64(Endian::deserialize_u64(bytes))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut i = 0;
        loop {
            if self.input[i] == b'\0' {
                break;
            }
            i += 1
        }
        let s =
            from_utf8(&self.input[..i]).map_err(|_| Error::ExpectedString)?;
        self.input = &self.input[i + 1..];
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let res = visitor.visit_bytes(self.input)?;
        Ok(res)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(TlvStruct::new(self))?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        use std::mem::size_of;

        match name {
            "string8" => {
                let s = self.read_tlv_string::<u8>()?;
                visitor.visit_borrowed_str(s)
            }
            "string16" => {
                let s = self.read_tlv_string::<u16>()?;
                visitor.visit_borrowed_str(s)
            }
            "string32" => {
                let s = self.read_tlv_string::<u32>()?;
                visitor.visit_borrowed_str(s)
            }
            "string64" => {
                let s = self.read_tlv_string::<u64>()?;
                visitor.visit_borrowed_str(s)
            }
            "vec8" => {
                let n = size_of::<u8>();
                let len = u8::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArray::new(self, len + 1))
            }
            "vec16" => {
                let n = size_of::<u16>();
                let len = u16::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArray::new(self, len + 1))
            }
            "vec32" => {
                let n = size_of::<u32>();
                let len = u32::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArray::new(self, len + 1))
            }
            "vec64" => {
                let n = size_of::<u64>();
                let len = u64::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArray::new(self, len + 1))
            }
            "vec8b" => {
                let n = size_of::<u8>();
                let len = u8::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArrayByteSized::new(self, len as usize))
            }
            "vec16b" => {
                let n = size_of::<u16>();
                let len = u16::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArrayByteSized::new(self, len as usize))
            }
            "vec32b" => {
                let n = size_of::<u32>();
                let len = u32::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArrayByteSized::new(self, len as usize))
            }
            "vec64b" => {
                let n = size_of::<u64>();
                let len = u64::read_size::<Endian>(&self.input[..n])?;
                self.input = &self.input[n..];
                visitor.visit_seq(PackedArrayByteSized::new(self, len as usize))
            }
            s => {
                unimplemented!("{}", s)
            }
        }
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    //TODO: however, enums actually work fine if the derive macro from
    //serde_repr is used, which crates the exact desired behavior, so perhaps
    //not a TODO
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

struct TlvStruct<'a, 'de: 'a, Endian: NumDe> {
    de: &'a mut Deserializer<'de, Endian>,
}

impl<'de, 'a, Endian: NumDe> TlvStruct<'a, 'de, Endian> {
    fn new(de: &'a mut Deserializer<'de, Endian>) -> Self {
        TlvStruct { de }
    }
}

impl<'de, 'a, Endian: NumDe> SeqAccess<'de> for TlvStruct<'a, 'de, Endian> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de).map(Some)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct_lv() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        version: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, b'm', b'u', b'f', b'f', b'i', b'n',
        b'\0',
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_str_lv8() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv8")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, 6, b'm', b'u', b'f', b'f', b'i',
        b'n',
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_str_lv16() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv16")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, 6, 0, b'm', b'u', b'f', b'f', b'i',
        b'n',
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_str_lv32() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv32")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, 6, 0, 0, 0, b'm', b'u', b'f', b'f',
        b'i', b'n',
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_str_lv64() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv64")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, b'm', b'u',
        b'f', b'f', b'i', b'n',
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_nested() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv64")]
        version: String,
        info: Info,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Info {
        typ: u8,
        version: u32,
        path: u64,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 99, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, b'm', b'u',
        b'f', b'f', b'i', b'n', 3, 57, 48, 0, 0, 254, 91, 10, 0, 0, 0, 0, 0,
    ];

    let expected = Version {
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
        info: Info {
            typ: 3,
            version: 12345,
            path: 678910,
        },
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv8() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv8")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 2, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv16() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv16")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 2, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv32() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv32")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 2, 0, 0, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv64() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv64")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 2, 0, 0, 0, 0, 0, 0, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv8b() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv8b")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 37, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv16b() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv16b")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 37, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv32b() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv32b")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 37, 0, 0, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}

#[test]
fn test_struct_vec_lv64b() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv64b")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let b = vec![
        47, 0, 0, 0, 9, 15, 0, 37, 0, 0, 0, 0, 0, 0, 0, // len
        // .1
        37, 0, 0, 0, 0, 0, 0, 0, // offset
        2, // typ
        9, 0, // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name
        // .2
        73, 0, 0, 0, 0, 0, 0, 0, // offset
        9, // typ
        6, 0, // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];

    let expected = Rreaddir {
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent {
                offset: 37,
                typ: 2,
                name: "blueberry".into(),
            },
            Dirent {
                offset: 73,
                typ: 9,
                name: "muffin".into(),
            },
        ],
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());
}
