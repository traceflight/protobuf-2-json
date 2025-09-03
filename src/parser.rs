//! Protobuf parser.

use std::ops::Range;

use base64::prelude::*;
use serde_json::{Map, Value, json};

use crate::{Field, FieldValue, Message, message::WireType, varint::decode_var};

const RESERVED_FIELD_NUMBER: Range<u64> = 19000..20000;

/// A protobuf parser that converts protobuf messages to JSON.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
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

    #[test]
    fn test_parse_3() {
        let data = hex!(
            "0a0a6173636f6e2d66756c6c120a6173636f6e2d66756c6c1a1b323032352d30392d30325430393a33373a32362e3033393032385a2203302e312a0474657374421b323032352d30392d30325430393a33373a32362e3033393032385a480068007205302e312e308a016e46756c6c204173636f6e20696d706c656d656e746174696f6e202868617368e280913235362c2041454144e280913132382077697468206e6f6e6365206d61736b696e67202620746167207472756e636174696f6e2c20584f46e280913132382c2043584f46e28091313238292e92012368747470733a2f2f6769746875622e636f6d2f6a6a6b756d2f6173636f6e2d66756c6c9a011a68747470733a2f2f646f63732e72732f6173636f6e2d66756c6ca2012368747470733a2f2f6769746875622e636f6d2f6a6a6b756d2f6173636f6e2d66756c6caa014612222f6170692f76312f6372617465732f6173636f6e2d66756c6c2f76657273696f6e731a202f6170692f76312f6372617465732f6173636f6e2d66756c6c2f6f776e657273"
        );
        let parser = Parser::new();
        let json = parser.parse(&data).unwrap();
        let expected = json!({
          "1": "ascon-full",
          "13": 0,
          "14": "0.1.0",
          "17": "Full Ascon implementation (hash‑256, AEAD‑128 with nonce masking & tag truncation, XOF‑128, CXOF‑128).",
          "18": "https://github.com/jjkum/ascon-full",
          "19": "https://docs.rs/ascon-full",
          "2": "ascon-full",
          "20": "https://github.com/jjkum/ascon-full",
          "21": {
            "2": "/api/v1/crates/ascon-full/versions",
            "3": "/api/v1/crates/ascon-full/owners"
          },
          "3": "2025-09-02T09:37:26.039028Z",
          "4": "0.1",
          "5": "test",
          "8": "2025-09-02T09:37:26.039028Z",
          "9": 0
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_parse_encoding_bytearray() {
        let data = hex!("4a050001020304");
        let parser = Parser::with_bytes_encoding(BytesEncoding::ByteArray);
        let json = parser.parse(&data).unwrap();
        let expected = json!({"9":[0,1,2,3,4]});
        assert_eq!(json, expected);
    }

    #[test]
    fn test_parse_num_array() {
        let data = hex!("4a050001020304");
        let parser = Parser::new();
        let json = parser.parse(&data).unwrap();
        let expected = json!({"9":"\u{0000}\u{0001}\u{0002}\u{0003}\u{0004}"});
        assert_eq!(json, expected);
    }
}
