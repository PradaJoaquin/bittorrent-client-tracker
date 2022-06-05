/// Represents a Bitfield.
///
/// It contains information about the pieces that the peer has.
#[derive(Debug)]
pub struct Bitfield {
    bitfield: Vec<u8>,
}

impl Bitfield {
    pub fn new(bitfield: Vec<u8>) -> Bitfield {
        Bitfield { bitfield }
    }

    /// Returns whether the bitfield has the piece with the given index.
    pub fn has_piece(&self, index: u32) -> bool {
        let byte = self.bitfield[index as usize / 8];
        let bit = byte >> (7 - (index % 8)) & 1;
        bit != 0
    }

    // Returns whether the bitfield has all the pieces.
    pub fn is_complete(&self) -> bool {
        self.bitfield.iter().all(|byte| *byte == 0b1111_1111)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitfield_has_all_pieces() {
        let bitfield = Bitfield::new(vec![0b11111111, 0b11111111, 0b11111111, 0b11111111]);

        assert!(bitfield.has_piece(4));
    }

    #[test]
    fn test_bitfield_has_one_piece() {
        let bitfield = Bitfield::new(vec![0b00000000, 0b00000010, 0b00000000, 0b00000000]);

        assert!(bitfield.has_piece(14));
    }

    #[test]
    fn test_bitfield_not_has_piece() {
        let bitfield = Bitfield::new(vec![0b11111111, 0b11111111, 0b11111101, 0b11111111]);

        assert!(!bitfield.has_piece(22));
    }
}
