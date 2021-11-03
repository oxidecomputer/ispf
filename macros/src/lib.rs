use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro_attribute]
pub fn with_length(attr: TokenStream, item: TokenStream) -> TokenStream {
    match attr.to_string().as_str() {
        "u16" => {
            let mut t = TokenStream::from_str(
                "#[serde(serialize_with = \"ipf::lv16::serialize\")]"
            ).unwrap();
            t.extend(item);
            t
        }
        _ => {
            panic!("with_length must be one of u8, u16, u32, u64")
        }
    }
}
