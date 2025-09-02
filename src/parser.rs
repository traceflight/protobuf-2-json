//! Protobuf parser.

use std::ops::Range;

use base64::prelude::*;
use serde_json::{Map, Value, json};

use crate::{Field, FieldValue, Message, message::WireType, varint::decode_var};

const RESERVED_FIELD_NUMBER: Range<u64> = 19000..20000;

/// A protobuf parser that converts protobuf messages to JSON.
#[derive(Debug, Default, Clone)]
pub struct Parser {
    /// How to encode bytes fields when converting to JSON.
    pub bytes_encoding: BytesEncoding,
}

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new parser with the given bytes encoding method.
    pub fn with_bytes_encoding(bytes_encoding: BytesEncoding) -> Self {
        Self { bytes_encoding }
    }

    /// Parse a protobuf message from the given byte slice and convert it to JSON.
    pub fn parse(&self, data: &[u8]) -> Option<Value> {
        self.parse_to_json(data, true)
    }

    /// Recursively parse a protobuf message and convert it to JSON.
    fn parse_to_json(&self, data: &[u8], first_layer: bool) -> Option<Value> {
        if data.is_empty() {
            return None;
        }

        // Check if the data is valid UTF-8 and not control characters
        let utf8_str = simdutf8::basic::from_utf8(data);
        if !first_layer && utf8_str.is_ok_and(|s| s.chars().all(|c| !c.is_control())) {
            return None;
        }

        let Message { fields, garbage } = self.parse_once(data);
        if fields.is_empty() {
            return None;
        }
        // If not the first layer, and the data is valid UTF-8 and contains garbage or reserved fields, return None
        if !first_layer
            && utf8_str.is_ok()
            && (garbage.is_some()
                || fields
                    .iter()
                    .any(|f| RESERVED_FIELD_NUMBER.contains(&f.number)))
        {
            return None;
        }

        let mut map = Map::new();
        for field in fields {
            let key = field.number.to_string();
            let value = match field.value {
                FieldValue::Varint(v) => Value::Number((v as usize).into()),
                FieldValue::Fixed64(v) => Value::Number(v.into()),
                FieldValue::Fixed32(v) => Value::Number(v.into()),
                FieldValue::LengthDelimited(bytes) => {
                    if let Some(nested) = self.parse_to_json(bytes, false) {
                        nested
                    } else {
                        match self.bytes_encoding {
                            BytesEncoding::Auto => {
                                if let Ok(s) = std::str::from_utf8(bytes) {
                                    Value::String(s.to_string())
                                } else {
                                    Value::String(BASE64_STANDARD.encode(bytes))
                                }
                            }
                            BytesEncoding::Base64 => Value::String(BASE64_STANDARD.encode(bytes)),
                            BytesEncoding::ByteArray => {
                                json!(bytes)
                            }
                            #[cfg(feature = "stfu8")]
                            BytesEncoding::Stfu8 => Value::String(stfu8::encode_u8(bytes)),
                            BytesEncoding::StringLossy => {
                                let s = String::from_utf8_lossy(bytes);
                                Value::String(s.to_string())
                            }
                        }
                    }
                }
                FieldValue::Invalid(_, _) | FieldValue::Incomplete(_, _) => match first_layer {
                    true => break,
                    false => return None,
                },
            };

            if let Some(existing) = map.get_mut(&key) {
                if let Value::Array(arr) = existing {
                    arr.push(value);
                } else {
                    let old_value = existing.clone();
                    *existing = Value::Array(vec![old_value, value]);
                }
            } else {
                map.insert(key, value);
            }
        }

        Some(Value::Object(map))
    }

    /// Parse a protobuf message from the given byte slice without recursion.
    pub fn parse_once<'a>(&self, mut data: &'a [u8]) -> Message<'a> {
        let mut msg = Message {
            fields: vec![],
            garbage: None,
        };

        let data = &mut data;

        loop {
            if data.is_empty() {
                break;
            }

            let tag = match decode_var(data) {
                Ok(tag) => tag,
                Err(_) => {
                    msg.garbage = Some(data);
                    break;
                }
            };

            let number = tag >> 3;
            let wire_type = WireType::from((tag & 0x07) as u8);

            let value = FieldValue::decode(data, wire_type);
            msg.fields.push(Field { number, value });
        }

        msg
    }
}

/// How to encode bytes fields when converting to JSON.
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BytesEncoding {
    #[default]
    /// Encode bytes as a string if valid UTF-8, otherwise as base64.
    Auto,

    /// Encode bytes as base64 string.
    Base64,

    /// Encode bytes as a JSON array of numbers.
    ByteArray,

    #[cfg(feature = "stfu8")]
    /// Encode bytes as [stfu8](https://crates.io/crates/stfu8) encoded string.
    Stfu8,

    /// Encode bytes as a UTF-8 lossy string.
    StringLossy,
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_parse_1() {
        let data = hex!("0d1c0000001203596f751a024d65202b2a0a0a066162633132331200");
        let parser = Parser::new();
        let json = parser.parse(&data).unwrap();
        let expected = json!({
            "1": 28,
            "2": "You",
            "3": "Me",
            "4": 43,
            "5": {
                "1": "abc123",
                "2": ""
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_parse_2() {
        let data =
            hex!("0d1c0000001203596f751a024d65202b2a0a0a06616263313233120031ba32a96cc10200003801");
        let parser = Parser::new();
        let json = parser.parse(&data).unwrap();
        let expected = json!({"1":28,"2":"You","3":"Me","4":43,"5":{"1":"abc123","2":""},"6":3029774971578u64,"7":1});
        assert_eq!(json, expected);
    }
}
