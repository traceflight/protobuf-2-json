use crate::decode_var;

/// Protocol buffer message.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Message<'a> {
    /// Decoded fields.
    pub fields: Vec<Field<'a>>,

    /// Garbage data at the end of the message.
    ///
    /// As opposed to an `UnknownValue::Invalid`, the garbage data did not have a valid field
    /// number and for that reason cannot be placed into the `fields` vector.
    pub garbage: Option<&'a [u8]>,
}

/// Decoded protocol buffer field.
///
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Field<'a> {
    /// Field number.
    pub number: u64,

    /// Decoded value.
    pub value: FieldValue<'a>,
}

/// Decoded protocol buffer value.
///
///
/// The wire type allows the decoder to tell how large an unknown value is. This allows the
/// unknown value to be skipped and decoding can continue from the next value.
///
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum FieldValue<'a> {
    /// Varint (wire type = 0).
    Varint(u128),

    /// 64-bit value (wire type = 1).
    Fixed64(u64),

    /// Length-delimited value (wire type = 2).
    LengthDelimited(&'a [u8]),

    /// 32-bit value (wire type = 5).
    Fixed32(u32),

    /// Invalid value.
    ///
    /// Invalid value is a value for which the wire type wasn't valid. Encountering invalid wire
    /// type will result in the remaining bytes to be consumed from the current variable length
    /// stream as it is imposible to tell how large such invalid value is.
    ///
    /// The decoding will continue after the current variable length value.
    Invalid(u8, &'a [u8]),

    /// Value which was incomplete due to missing bytes in the payload.
    Incomplete(WireType, &'a [u8]),
}

impl<'a> FieldValue<'a> {
    pub fn decode(data: &mut &'a [u8], wire_type: WireType) -> Self {
        match wire_type {
            WireType::Varint => match decode_var(data) {
                Ok(v) => FieldValue::Varint(v as u128),
                Err(_) => FieldValue::Incomplete(wire_type, *data),
            },
            WireType::Fixed64 => {
                if data.len() < 8 {
                    FieldValue::Incomplete(wire_type, *data)
                } else {
                    let (num_bytes, rest) = data.split_at(8);
                    *data = rest;
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(num_bytes);
                    FieldValue::Fixed64(u64::from_le_bytes(arr))
                }
            }
            WireType::LengthDelimited => match decode_var(data) {
                Ok(len) => {
                    let len = len as usize;
                    if data.len() < len {
                        FieldValue::Incomplete(wire_type, *data)
                    } else {
                        let (bytes, rest) = data.split_at(len);
                        *data = rest;
                        FieldValue::LengthDelimited(bytes)
                    }
                }
                Err(_) => FieldValue::Incomplete(wire_type, *data),
            },
            WireType::Fixed32 => {
                if data.len() < 4 {
                    FieldValue::Incomplete(wire_type, *data)
                } else {
                    let (num_bytes, rest) = data.split_at(4);
                    *data = rest;
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(num_bytes);
                    FieldValue::Fixed32(u32::from_le_bytes(arr))
                }
            }
            WireType::Invalid(wt) => FieldValue::Invalid(wt, *data),
        }
    }
}

/// Protocol buffer wire types.
#[derive(Debug, PartialEq, Clone, Eq, Copy, Hash)]
#[repr(u8)]
pub enum WireType {
    /// Varint (0)
    Varint = 0,

    /// 64-bit (1)
    Fixed64 = 1,

    /// Length-delimited (2)
    LengthDelimited = 2,

    /// 32-bit (5)
    Fixed32 = 5,

    /// Invalid wire type
    Invalid(u8),
}

impl From<u8> for WireType {
    fn from(value: u8) -> Self {
        match value {
            0 => WireType::Varint,
            1 => WireType::Fixed64,
            2 => WireType::LengthDelimited,
            5 => WireType::Fixed32,
            other => WireType::Invalid(other),
        }
    }
}
