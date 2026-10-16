pub use buffa;
pub use buffa::{EnumValue, Enumeration, Message, MessageField};
pub use buffa_types::google::protobuf as wkt;
pub use connectrpc;

connectrpc::include_generated!();

include!(concat!(env!("OUT_DIR"), "/local_api.rs"));

pub trait KnownEnum: Enumeration {
    fn known(value: impl Into<EnumValue<Self>>) -> Option<Self> {
        value.into().as_known()
    }
}

impl<E: Enumeration> KnownEnum for E {}

pub mod json {
    use buffa::json::{JsonParseOptions, with_json_parse_options};
    use serde::de::{DeserializeOwned, Error as _};
    use serde_json::Value;

    pub fn lenient<T>(parse: impl FnOnce() -> T) -> T {
        with_json_parse_options(
            &JsonParseOptions::new().ignore_unknown_enum_values(true),
            parse,
        )
    }

    pub fn from_value<T: DeserializeOwned>(value: Value) -> serde_json::Result<T> {
        if !value.is_object() {
            return Err(serde_json::Error::custom("a message is a JSON object"));
        }
        lenient(|| serde_json::from_value(value))
    }

    pub fn from_str<T: DeserializeOwned>(text: &str) -> serde_json::Result<T> {
        from_value(serde_json::from_str(text)?)
    }

    pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> serde_json::Result<T> {
        from_value(serde_json::from_slice(bytes)?)
    }
}
