//! # protobuf-to-json
//!
//! A parser that converts arbitrary protobuf data to json
//!
//! ## Features
//! * No schema required
//! * Use field number as json key
//! * Configurable bytes encoding (base64, hex, byte array, etc.)
//! * Automatically guesses length-delimited value types (string, nested message, bytes)
//!
//! ## Limitations
//! * Length-delimited value type is guessed based on content. It may not always be correct.
//! * Repeated fields (with the same field number) may not be grouped into arrays when only one field is parsed.
//!
//! ## Examples
//!
//! ``` rust
//! use protobuf_to_json::Parser;
//! use hex_literal::hex;
//! use serde_json::json;
//!
//! let data = hex!("0d1c0000001203596f751a024d65202b2a0a0a066162633132331200");
//! let parser = Parser::new();
//! let json = parser.parse(&data).unwrap();
//! let expected = json!({
//!     "1": 28,
//!     "2": "You",
//!     "3": "Me",
//!     "4": 43,
//!     "5": {
//!         "1": "abc123",
//!         "2": ""
//!     }
//! });
//! assert_eq!(json, expected);
//! ```
//!

mod message;
mod parser;
mod varint;

pub use message::{Field, FieldValue, Message};
pub use parser::{BytesEncoding, Parser};
pub use varint::decode_var;
