use std::marker::PhantomData;
use std::convert::TryInto;
use std::str::from_utf8;
use std::fmt;

use serde::Deserialize;
use serde::de::{
    self, 
    SeqAccess,
    DeserializeSeed, 
    Visitor,
};
use crate::LittleEndian;

pub trait NumDe {
    fn deserialize_u16(v: [u8; 2]) -> u16;
    fn deserialize_u32(v: [u8; 4]) -> u32;
    fn deserialize_u64(v: [u8; 8]) -> u64;
}

impl NumDe for LittleEndian {
    fn deserialize_u16(v: [u8; 2]) -> u16 { u16::from_le_bytes(v) }
    fn deserialize_u32(v: [u8; 4]) -> u32 { u32::from_le_bytes(v) }
    fn deserialize_u64(v: [u8; 8]) -> u64 { u64::from_le_bytes(v) }
}

trait ReadSize {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize>;
}

impl ReadSize for u8 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        match bytes.get(0) {
            Some(x) => Ok(*x as usize),
            None => Err(Error::ExpectedInteger),
        }
    }
}

impl ReadSize for u16 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u16(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?
        ) as usize)
    }
}

impl ReadSize for u32 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u32(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?
        ) as usize)
    }
}

impl ReadSize for u64 {
    fn read_size<Endian: NumDe>(bytes: &[u8]) -> Result<usize> {
        Ok(Endian::deserialize_u64(
            bytes.try_into().map_err(|_| Error::ExpectedInteger)?
        ) as usize)
    }
}

use crate::error::{Error, Result};

pub struct Deserializer<'de, Endian: NumDe> {
    input: &'de [u8],
    endian: PhantomData::<Endian>,
}

impl<'de, Endian: NumDe> Deserializer<'de, Endian> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { 
            input,
            endian: PhantomData::<Endian>{},
        }
    }

    fn read_tlv_string<T: ReadSize>(&mut self) -> Result<&'de str> {
        use std::mem::size_of;

        let n = size_of::<T>();

        let len = T::read_size::<Endian>(&self.input[..n])?;
        let s = from_utf8(&self.input[n..n+len])
            .map_err(|_| Error::Eof)?;

        self.input = &self.input[n+len..];
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
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        println!("TRAILING: {}", deserializer.input.len());
        Err(Error::TrailingBytes)
    }
}

pub struct TlvStringVisitor;
impl <'de> Visitor<'de> for TlvStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string prifixed by a 16 bit length")
    }

    fn visit_borrowed_str<E>(self, value: &'de str)
    -> core::result::Result<Self::Value, E> {
        Ok(value.to_string())
    }
}

impl<'de, 'a, Endian: NumDe>
de::Deserializer<'de> for &'a mut Deserializer<'de, Endian> {
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
        let s = from_utf8(&self.input[..i]).map_err(|_| Error::ExpectedString)?;
        self.input = &self.input[i+1..];
        let res = visitor.visit_borrowed_str(s);
        res
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
            _ => {
                unimplemented!()
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

    //TODO
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
        TlvStruct{ de }
    }
}

impl<'de, 'a, Endian: NumDe>
SeqAccess<'de> for TlvStruct<'a, 'de, Endian> {

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
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n', b'\0',
    ];

    let expected = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());

}

#[test]
fn test_struct_lv8() {

    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::lv8")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    let expected = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());

}

#[test]
fn test_struct_lv16() {

    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::lv16")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    let expected = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());

}

#[test]
fn test_struct_lv32() {

    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::lv32")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    let expected = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());

}

#[test]
fn test_struct_lv64() {

    #[derive(Deserialize, PartialEq, Debug)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::lv64")]
        version: String,
    }

    let b = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0, 0, 0, 0, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    let expected = Version{
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
        #[serde(with = "crate::lv64")]
        version: String,
        info: Info,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Info {
        typ: u8,
        version: u32,
        path: u64
    }

    let b = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0, 0, 0, 0, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
        3,
        57, 48, 0, 0,
        254, 91, 10, 0, 0, 0, 0, 0,
    ];

    let expected = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
        info: Info{
            typ: 3,
            version: 12345,
            path: 678910,
        }
    };

    assert_eq!(expected, from_bytes_le(b.as_slice()).unwrap());

}
