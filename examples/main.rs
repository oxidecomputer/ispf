// Copyright 2021 Oxide Computer Company

use ipf::{to_bytes_le, from_bytes_le};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Version {
    size: u32,
    typ: u8,
    tag: u16,
    msize: u32,
    #[serde(with = "ipf::str_lv64")]
    version: String,
}

fn main() -> Result<(), ipf::Error>{

    let v = Version{
        size: 47,
        typ: 9,
        tag: 15,
        msize: 99,
        version: "muffin".into(),
    };

    let out = to_bytes_le(&v)?;
    println!("{:?}", out);

    let full_circle: Version = from_bytes_le(out.as_slice())?;
    println!("{:#?}", full_circle);
    assert_eq!(v, full_circle);

    Ok(())

}
