//! # protobuf-to-json
//!
//! A converter that parses arbitrary protobuf data to json
//!
//! ## Features
//! * No schema required
//! * Use field number as json key
//! * Configurable bytes encoding (base64, hex, byte array, etc.)
//! * Guess length-delimited value types (string, nested message, bytes)
//!
//! ## Limitations
//! * Length-delimited value type is guessed based on content. It may not always be correct.
//! * Repeated fields (with the same field number) may not be grouped into arrays when only one field is parsed.
//!
//! ## Examples
//!
//!

mod message;
mod parser;
mod varint;

pub use message::{Field, FieldValue, Message};
pub use parser::{BytesEncoding, Parser};
pub use varint::decode_var;
