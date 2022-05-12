#[derive(PartialEq, Debug)]
pub enum UrlEncoderError {
    InvalidUrlEncode,
}

/// Takes an hex string and applies Percent-Encoding, returning an encoded version.
///
/// # Example
///
/// ```rust
/// use bit_torrent_rustico::encoder_decoder::url_encoder::encode;
///
/// let hex_string = "2c6b6858d61da9543d4231a71db4b1c9264b0685";
/// let encoded_hex_string = encode(hex_string);
///
/// assert_eq!(encoded_hex_string, "%2c%6b%68%58%d6%1d%a9%54%3d%42%31%a7%1d%b4%b1%c9%26%4b%06%85");
/// ```
pub fn encode(hex_string: &str) -> String {
    if hex_string.is_empty() {
        return hex_string.to_string();
    }
    let mut encoded_hex_string = hex_string
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("%");
    encoded_hex_string.insert(0, '%');
    encoded_hex_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty_string_returns_empty_string() {
        assert_eq!("", encode(""));
    }

    #[test]
    fn test_encode_info_hash() {
        let info_hash = "2c6b6858d61da9543d4231a71db4b1c9264b0685";
        let expected_info_hash = "%2c%6b%68%58%d6%1d%a9%54%3d%42%31%a7%1d%b4%b1%c9%26%4b%06%85";

        assert_eq!(expected_info_hash, encode(info_hash));
    }
}
