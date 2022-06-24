use std::collections::HashMap;

use crate::torrent_handler::status::PieceStatus;

/// Represents a Bitfield.
///
/// It contains information about the pieces that the peer has.
#[derive(Debug)]
pub struct Bitfield {
    pub bitfield: Vec<u8>,
}

impl Bitfield {
    pub fn new(bitfield: Vec<u8>) -> Bitfield {
        Bitfield { bitfield }
    }

    /// Returns whether the bitfield has the piece with the given index.
    pub fn has_piece(&self, index: u32) -> bool {
        let byte_index = (index / 8) as usize;
        let byte = self.bitfield[byte_index];

        let bit_index = 7 - (index % 8); // Gets the bit index in the byte (from the right)

        // Moves the corresponding bit to the rightmost side of the byte
        // and then checks if that last bit is 1 or 0
        let bit = (byte >> bit_index) & 1;
        bit != 0
    }

    // Returns whether the bitfield has all the pieces.
    pub fn is_complete(&self) -> bool {
        self.bitfield.iter().all(|byte| *byte == 0b1111_1111)
    }

    /// Creates a bitfield from pieces status
    pub fn from(pieces_status: &HashMap<u32, PieceStatus>) -> Bitfield {
        let bytes_count = (pieces_status.len() + 7) / 8;
        let mut bitfield = vec![0; bytes_count];

        for (piece_index, status) in pieces_status {
            if status == &PieceStatus::Finished {
                let byte_index = (piece_index / 8) as usize;
                let byte = bitfield[byte_index];

                let bit_index = 7 - (piece_index % 8); // Gets the bit index in the byte (from the right)
                let bit = 1 << bit_index; // Shifts 1 to the left bit_index times

                bitfield[byte_index] = byte | bit;
            }
        }

        Self::new(bitfield)
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

    #[test]
    fn test_bitfield_from_one_piece_finished() {
        let mut pieces_status = HashMap::new();
        for i in 0..8 {
            pieces_status.insert(i, PieceStatus::Free);
        }

        pieces_status.insert(0, PieceStatus::Finished);

        let bitfield = Bitfield::from(&pieces_status);

        assert_eq!(bitfield.bitfield, vec![0b1000_0000]);
    }

    #[test]
    fn test_bitfield_from_one_piece_finished_in_the_middle() {
        let mut pieces_status = HashMap::new();
        for i in 0..8 {
            pieces_status.insert(i, PieceStatus::Free);
        }

        pieces_status.insert(3, PieceStatus::Finished);

        let bitfield = Bitfield::from(&pieces_status);

        assert_eq!(bitfield.bitfield, vec![0b0001_0000]);
    }

    #[test]
    fn test_bitfield_from_all_pieces_finished() {
        let mut pieces_status = HashMap::new();
        for i in 0..8 {
            pieces_status.insert(i, PieceStatus::Finished);
        }

        let bitfield = Bitfield::from(&pieces_status);

        assert_eq!(bitfield.bitfield, vec![0b1111_1111]);
    }

    #[test]
    fn test_from_two_bytes() {
        let mut pieces_status = HashMap::new();
        for i in 0..9 {
            pieces_status.insert(i, PieceStatus::Finished);
        }

        let bitfield = Bitfield::from(&pieces_status);

        assert_eq!(bitfield.bitfield, vec![0b1111_1111, 0b1000_0000]);
    }

    #[test]
    fn test_from_two_bytes_complete() {
        let mut pieces_status = HashMap::new();
        for i in 0..16 {
            pieces_status.insert(i, PieceStatus::Finished);
        }

        let bitfield = Bitfield::from(&pieces_status);

        assert_eq!(bitfield.bitfield, vec![0b1111_1111, 0b1111_1111]);
    }
}
