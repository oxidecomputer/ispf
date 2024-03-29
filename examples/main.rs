// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

use ispf::{from_bytes_le, to_bytes_le};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Version {
    size: u32,
    typ: u8,
    tag: u16,
    msize: u32,
    #[serde(with = "ispf::str_lv64")]
    version: String,
}

fn main() -> Result<(), ispf::Error> {
    let v = Version {
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
