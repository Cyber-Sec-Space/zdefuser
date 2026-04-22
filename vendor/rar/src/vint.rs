use std::num::NonZero;

/// Returns the next complete vint number as u64.
pub fn vint(input: &[u8]) -> nom::IResult<&[u8], u64> {
    if input.is_empty() {
        return Err(nom::Err::Incomplete(nom::Needed::Size(
            NonZero::new(1).unwrap(),
        )));
    }

    let mut len = 0;
    let mut found_end = false;

    // Find the length by counting high bits
    for (i, &byte) in input.iter().enumerate() {
        len = i + 1;
        if (byte & 0x80) == 0 {
            found_end = true;
            break;
        }
        if i >= 8 {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge,
            )));
        }
    }

    // If we didn't find the end byte, we need more data
    if !found_end {
        return Err(nom::Err::Incomplete(nom::Needed::Size(
            NonZero::new(1).unwrap(),
        )));
    }

    // Extract the value using the original RAR vint algorithm
    // Start with the final byte (no high bit), then add high-bit bytes in reverse
    let mut out: u64 = input[len - 1] as u64; // Final byte contributes full value

    // Add the high-bit bytes in reverse order
    for i in (0..len - 1).rev() {
        out <<= 7;
        out |= (input[i] & 0x7F) as u64;
    }

    Ok((&input[len..], out))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if a byte has the vint bit set
    fn is_vint_bit(input: u8) -> bool {
        (input & 0x80) != 0
    }

    /// Split a vint byte into its components
    fn split_vint(input: u8) -> (bool, u8) {
        (is_vint_bit(input), input & 0x7F)
    }

    #[test]
    fn test_vint() {
        let data = [0x01];
        let result = vint(&data);
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_split_vint() {
        let result = split_vint(0x80);
        assert_eq!(result, (true, 0));

        let result = split_vint(0x7F);
        assert_eq!(result, (false, 0x7F));
    }

    #[test]
    fn test_is_vint_bit() {
        assert!(is_vint_bit(0x80));
        assert!(!is_vint_bit(0x7F));
    }
}
