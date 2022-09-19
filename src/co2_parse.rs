use std::{error::Error, fmt::Display};

/// Decodes a data packet from the CO2 monitor.
fn decode(encoded: [u8; 8]) -> [u8; 8] {
    let mut buf: [u8; 8] = encoded;

    buf.swap(0, 2);
    buf.swap(1, 4);
    buf.swap(3, 7);
    buf.swap(5, 6);

    // Other CO2 monitors seem to XOR the data with hardcoded key. The
    // one that I have doesn't. Leave the code here in case it becomes
    // useful again.
    let key: [u8; 8] = [0; 8];
    for i in 0..8 {
        buf[i] ^= key[i];
    }

    let mut result: [u8; 8] = [0; 8];

    result[7] = (buf[6] << 5) | (buf[7] >> 3);
    result[6] = (buf[5] << 5) | (buf[6] >> 3);
    result[5] = (buf[4] << 5) | (buf[5] >> 3);
    result[4] = (buf[3] << 5) | (buf[4] >> 3);
    result[3] = (buf[2] << 5) | (buf[3] >> 3);
    result[2] = (buf[1] << 5) | (buf[2] >> 3);
    result[1] = (buf[0] << 5) | (buf[1] >> 3);
    result[0] = (buf[7] << 5) | (buf[0] >> 3);

    // "Htemp99e"
    let magic_word: [u8; 8] = [0x48, 0x74, 0x65, 0x6d, 0x70, 0x39, 0x39, 0x65];

    for i in 0..8 {
        result[i] = result[i].wrapping_sub((magic_word[i] << 4) | (magic_word[i] >> 4));
    }

    result
}

/// Checks whether the checksum of a data packet is correct.
fn has_ok_checksum(buf: [u8; 8]) -> bool {
    buf[0..3].iter().fold(0u8, |acc, &e| acc.wrapping_add(e)) == buf[3]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    Co2Level { ppm: u16 },
    Temperature { celsius: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidChecksum,
    UnrecognizedMessage,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidChecksum => write!(f, "Data failed checksum check"),
            ParseError::UnrecognizedMessage => write!(f, "Data was not recognized"),
        }
    }
}

impl Error for ParseError {}

impl TryFrom<&[u8; 8]> for Message {
    type Error = ParseError;

    fn try_from(value: &[u8; 8]) -> Result<Self, Self::Error> {
        let decoded = decode(*value);
        if !has_ok_checksum(decoded) {
            return Err(ParseError::InvalidChecksum);
        }

        if decoded[4] != 0x0d {
            return Err(ParseError::UnrecognizedMessage);
        }

        let w = (u16::from(decoded[1]) << 8) | u16::from(decoded[2]);

        match decoded[0] {
            0x42 => Ok(Message::Temperature {
                celsius: f32::from(w) * 0.0625 - 273.15,
            }),
            0x50 => Ok(Message::Co2Level { ppm: w }),
            _ => Err(ParseError::UnrecognizedMessage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_works() {
        assert_eq!(
            decode([183, 164, 50, 182, 200, 154, 156, 80]),
            [66, 18, 192, 20, 13, 0, 0, 0]
        );
    }

    #[test]
    fn checksum_works() {
        assert!(has_ok_checksum([66, 18, 192, 20, 13, 0, 0, 0]));
    }
}
