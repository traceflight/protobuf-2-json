//! Variable-length integer decoding.
//!

/// Most-significant byte, == 0x80
pub const MSB: u8 = 0b1000_0000;
/// All bits except for the most significant. Can be used as bitmask to drop the most-signficant
/// bit using `&` (binary-and).
const DROP_MSB: u8 = 0b0111_1111;

/// Decode a variable-length integer from a byte slice.
pub fn decode_var(src: &mut &[u8]) -> Result<u64, ()> {
    let mut result: u64 = 0;
    let mut shift = 0;

    let mut success = false;
    for b in src.iter() {
        let msb_dropped = b & DROP_MSB;
        result |= (msb_dropped as u64) << shift;
        shift += 7;

        if b & MSB == 0 || shift > (9 * 7) {
            success = b & MSB == 0;
            break;
        }
    }

    if success {
        *src = &src[shift / 7..];
        Ok(result)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_max_u64() {
        let max_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(
            decode_var(&mut max_vec_encoded.as_slice()).unwrap(),
            u64::max_value()
        );
    }

    #[test]
    fn test_decode_zero() {
        let zero_encoded = vec![0x00];
        assert_eq!(decode_var(&mut zero_encoded.as_slice()).unwrap(), 0);
    }

    #[test]
    fn test_decode_one() {
        let one_encoded = vec![0x01];
        assert_eq!(decode_var(&mut one_encoded.as_slice()).unwrap(), 1);
    }

    #[test]
    fn test_decode_large_number() {
        let large_number_encoded = vec![0xAC, 0x02];
        assert_eq!(
            decode_var(&mut large_number_encoded.as_slice()).unwrap(),
            300
        );
    }

    #[test]
    fn test_decode_incomplete_sequence() {
        let incomplete_encoded = vec![0xFF, 0xFF, 0xFF];
        assert!(decode_var(&mut incomplete_encoded.as_slice()).is_err());
    }

    #[test]
    fn test_decode_single_byte_with_msb() {
        let single_byte_with_msb = vec![0x80];
        assert!(decode_var(&mut single_byte_with_msb.as_slice()).is_err());
    }

    #[test]
    fn test_decode_empty_input() {
        let empty_input: Vec<u8> = vec![];
        assert!(decode_var(&mut empty_input.as_slice()).is_err());
    }
}
