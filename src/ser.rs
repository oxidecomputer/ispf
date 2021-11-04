use serde::{ser, Serialize};
use std::marker::PhantomData;

use crate::LittleEndian;
use crate::error::{Error, Result};

pub trait NumSer {
    fn serialize_u16(v: u16) -> [u8; 2];
    fn serialize_u32(v: u32) -> [u8; 4];
    fn serialize_u64(v: u64) -> [u8; 8];
}

impl NumSer for LittleEndian {
    fn serialize_u16(v: u16) -> [u8; 2] { v.to_le_bytes() }
    fn serialize_u32(v: u32) -> [u8; 4] { v.to_le_bytes() }
    fn serialize_u64(v: u64) -> [u8; 8] { v.to_le_bytes() }
}

pub struct Serializer<Endian: NumSer> {
    output: Vec<u8>,
    endian: PhantomData::<Endian>,
}

pub fn to_bytes_le<T>(value: &T) -> Result<Vec::<u8>> 
where
    T: Serialize
{
    to_bytes::<LittleEndian, T>(value)
}

pub fn to_bytes<Endian, T>(value: &T) -> Result<Vec::<u8>>
where
    T: Serialize,
    Endian: NumSer
{
    let mut serializer = Serializer{
        output: Vec::new(),
        endian: PhantomData::<Endian>{},
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a, Endian: NumSer> ser::Serializer for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(self.output.push(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(self.output.extend_from_slice(&Endian::serialize_u16(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(self.output.extend_from_slice(&Endian::serialize_u32(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(self.output.extend_from_slice(&Endian::serialize_u64(v)))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.output.extend_from_slice(v.as_bytes());
        self.output.push(0); //default is null terminated
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(
        self,
        _value: &T
    ) -> Result<Self::Ok>
    where
        T: Serialize
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str
    ) -> Result<Self::Ok>
    {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str
    ) -> Result<Self::Ok> {
        println!("{} {} {}", _name, _variant_index, _variant);
        unimplemented!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T
    ) -> Result<Self::Ok>
    where
        T: Serialize
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T
    ) -> Result<Self::Ok>
    where
        T: Serialize
    {
        unimplemented!()
    }

    fn serialize_seq(
        self,
        _len: Option<usize>
    ) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(
        self,
        _len: usize
    ) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize
    ) -> Result<Self::SerializeTupleStruct>{
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(
        self,
        _len: Option<usize>
    ) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }

}

impl<'a, Endian: NumSer> ser::SerializeSeq for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }

}

impl<'a, Endian: NumSer> ser::SerializeTuple for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }

}

impl<'a, Endian: NumSer>
ser::SerializeTupleStruct for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }

}

impl<'a, Endian: NumSer>
ser::SerializeTupleVariant for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }

}

impl<'a, Endian: NumSer> ser::SerializeMap for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a, Endian: NumSer>
ser::SerializeStruct for &'a mut Serializer<Endian> {

    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T)
    -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }

}

impl<'a, Endian: NumSer>
ser::SerializeStructVariant for &'a mut Serializer<Endian> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T)
    -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct_lv() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        version: String,
    }

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n', b'\0',
    ];

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_struct_str_lv8() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(serialize_with = "crate::str_lv8::serialize")]
        version: String,
    }

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_struct_str_lv16() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(with = "crate::str_lv16")]
        version: String,
    }

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_struct_str_lv32() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(serialize_with = "crate::str_lv32::serialize")]
        version: String,
    }

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_struct_str_lv64() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(serialize_with = "crate::str_lv64::serialize")]
        version: String,
    }

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        99, 0, 0, 0,
        6, 0, 0, 0, 0, 0, 0, 0,
        b'm', b'u', b'f', b'f', b'i', b'n',
    ];

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_nested_struct() {

    #[derive(Serialize)]
    struct Version {
        size: u32,
        typ: u8,
        tag: u16,
        msize: u32,
        #[serde(serialize_with = "crate::str_lv64::serialize")]
        version: String,
        info: Info,
    }

    #[derive(Serialize)]
    struct Info {
        typ: u8,
        version: u32,
        path: u64
    }

    let v = Version{
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

    let expected = vec![
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

    assert_eq!(to_bytes_le(&v).unwrap(), expected);

}

#[test]
fn test_struct_vec_lv8() {

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv8")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let r = Rreaddir{
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent{offset: 37, typ: 2, name: "blueberry".into()},
            Dirent{offset: 73, typ: 9, name: "muffin".into()},
        ]
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        2, // len
        
        // .1
        37, 0, 0, 0, 0, 0, 0, 0,                              // offset
        2,                                                    // typ
        9, 0,                                                 // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name

        // .2
        73, 0, 0, 0, 0, 0, 0, 0,            // offset
        9,                                  // typ
        6, 0,                               // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];


    assert_eq!(to_bytes_le(&r).unwrap(), expected);

}

#[test]
fn test_struct_vec_lv16() {

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv16")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let r = Rreaddir{
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent{offset: 37, typ: 2, name: "blueberry".into()},
            Dirent{offset: 73, typ: 9, name: "muffin".into()},
        ]
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        2, 0, // len
        
        // .1
        37, 0, 0, 0, 0, 0, 0, 0,                              // offset
        2,                                                    // typ
        9, 0,                                                 // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name

        // .2
        73, 0, 0, 0, 0, 0, 0, 0,            // offset
        9,                                  // typ
        6, 0,                               // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];


    assert_eq!(to_bytes_le(&r).unwrap(), expected);

}

#[test]
fn test_struct_vec_lv32() {

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv32")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let r = Rreaddir{
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent{offset: 37, typ: 2, name: "blueberry".into()},
            Dirent{offset: 73, typ: 9, name: "muffin".into()},
        ]
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        2, 0, 0, 0, // len
        
        // .1
        37, 0, 0, 0, 0, 0, 0, 0,                              // offset
        2,                                                    // typ
        9, 0,                                                 // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name

        // .2
        73, 0, 0, 0, 0, 0, 0, 0,            // offset
        9,                                  // typ
        6, 0,                               // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];


    assert_eq!(to_bytes_le(&r).unwrap(), expected);

}

#[test]
fn test_struct_vec_lv64() {

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Rreaddir {
        pub size: u32,
        pub typ: u8,
        pub tag: u16,
        #[serde(with = "crate::vec_lv64")]
        pub data: Vec<Dirent>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Dirent {
        pub offset: u64,
        pub typ: u8,
        #[serde(with = "crate::str_lv16")]
        pub name: String,
    }

    let r = Rreaddir{
        size: 47,
        typ: 9,
        tag: 15,
        data: vec![
            Dirent{offset: 37, typ: 2, name: "blueberry".into()},
            Dirent{offset: 73, typ: 9, name: "muffin".into()},
        ]
    };

    let expected = vec![
        47, 0, 0, 0,
        9,
        15, 0,
        2, 0, 0, 0, 0, 0, 0, 0, // len
        
        // .1
        37, 0, 0, 0, 0, 0, 0, 0,                              // offset
        2,                                                    // typ
        9, 0,                                                 // name.len
        b'b', b'l', b'u', b'e', b'b', b'e', b'r', b'r', b'y', // name

        // .2
        73, 0, 0, 0, 0, 0, 0, 0,            // offset
        9,                                  // typ
        6, 0,                               // name.len
        b'm', b'u', b'f', b'f', b'i', b'n', //name
    ];


    assert_eq!(to_bytes_le(&r).unwrap(), expected);

}
