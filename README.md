# Internet-Style Packet Format (IPF) for Serde

This crate provides machinery for serializing and deserializing Internet-style
packets with [Serde](https://serde.rs).

## By Example

Consider the following Internet-style packet

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                              size                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    version    |              tag              |     msize     :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
:                                               |  version.size :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
:               |                   version                     :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
:                               ...                             :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

```

with Serde/IPF this can be represented as

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Version {
    size: u32,
    typ: u8,
    tag: u16,
    msize: u32,
    #[serde(with = "ipf::str_lv16")]
    version: String,
}
```

The annotation on `version` in this data structure indicates that `version`
should be serialized as a length-value pair with a 16-bit unsigned integer as
the length.

A basic round trip to and from wire format is perfomred as the following
([full example](examples/main.rs)).

```rust
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
```

So the basic thing that IPF does is create packed wire representations for RUst
data types and provides a cofigurable means by which to represent types that are
not statically sized, such as the `ipf::str_lv64` serializer annotation above.

## Available Formatters

### Strings

- `str_lv8`
- `str_lv16`
- `str_lv32`
- `str_lv64`

Length value represents the number of total bytes in the string.

### Vectors by count

- `vec_lv8`
- `vec_lv16`
- `vec_lv32`
- `vec_lv64`

Length value represents number of elements in the vector.

### Vectors by bytes

- `vec_lv8b`
- `vec_lv16b`
- `vec_lv32b`
- `vec_lv64b`

Length value represents number of total bytes in the vector (the sume of the
size of the elements).

## Building

```
cargo build
cargo test
```
