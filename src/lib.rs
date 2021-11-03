mod de;
mod error;
mod ser;

pub use de::{from_bytes, from_bytes_le, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, to_bytes_le, Serializer};

pub struct LittleEndian { }

pub mod lv8 {
    use serde::ser::SerializeTuple;

    pub fn serialize<S>(v: &str, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut t = s.serialize_tuple(std::mem::size_of::<u8>()+v.len())?;
        t.serialize_element(&(v.len() as u8))?;
        t.serialize_element(v.as_bytes())?;
        t.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        d.deserialize_tuple_struct("string8", 2, crate::de::TlvStringVisitor)
    }
}

pub mod lv16 {
    use serde::ser::SerializeTuple;

    pub fn serialize<S>(v: &str, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut t = s.serialize_tuple(std::mem::size_of::<u16>()+v.len())?;
        t.serialize_element(&(v.len() as u16))?;
        t.serialize_element(v.as_bytes())?;
        t.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        d.deserialize_tuple_struct("string16", 2, crate::de::TlvStringVisitor)
    }

}

pub mod lv32 {
    use serde::ser::SerializeTuple;

    pub fn serialize<S>(v: &str, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut t = s.serialize_tuple(std::mem::size_of::<u32>()+v.len())?;
        t.serialize_element(&(v.len() as u32))?;
        t.serialize_element(v.as_bytes())?;
        t.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        d.deserialize_tuple_struct("string32", 2, crate::de::TlvStringVisitor)
    }
}

pub mod lv64 {
    use serde::ser::SerializeTuple;

    pub fn serialize<S>(v: &str, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut t = s.serialize_tuple(std::mem::size_of::<u64>()+v.len())?;
        t.serialize_element(&(v.len() as u64))?;
        t.serialize_element(v.as_bytes())?;
        t.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        d.deserialize_tuple_struct("string64", 2, crate::de::TlvStringVisitor)
    }
}