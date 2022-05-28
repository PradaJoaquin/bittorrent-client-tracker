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

    /// Returns whether the peer has the piece with the given index.
    pub fn has_piece(&self, index: u32) -> bool {
        let byte = self.bitfield[index as usize / 8];
        let bit = byte >> (7 - (index % 8)) & 1;
        bit != 0
    }
}

/// Represents the payload of a Request message.
#[derive(Debug)]
pub struct Request {
    index: u32,
    begin: u32,
    length: u32,
}

impl Request {
    /// Creates a new `Request` message.
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index,
            begin,
            length,
        }
    }

    /// Converts a `Request` message to a byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; 12];
        bytes[0..4].copy_from_slice(&self.index.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.begin.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.length.to_be_bytes());
        bytes
    }
}

// IDs of the messages defined in the protocol.
#[derive(PartialEq, Debug, Clone)]
pub enum MessageId {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
    Port = 9,
}

/// The message that is sent to the peer.
///
/// It contains the message ID and the payload.
#[derive(Debug)]
pub struct Message {
    pub id: MessageId,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum FromMessageError {
    InvalidMessage,
}

impl Message {
    /// Creates a new `Message` from a message ID and a payload.
    pub fn new(id: MessageId, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }

    /// Parses a byte array into a `Message`.
    pub fn from_bytes(msg_type: [u8; 1], payload: &[u8]) -> Result<Self, FromMessageError> {
        let id = match msg_type[0] {
            0 => MessageId::Choke,
            1 => MessageId::Unchoke,
            2 => MessageId::Interested,
            3 => MessageId::NotInterested,
            4 => MessageId::Have,
            5 => MessageId::Bitfield,
            6 => MessageId::Request,
            7 => MessageId::Piece,
            8 => MessageId::Cancel,
            9 => MessageId::Port,
            _ => return Err(FromMessageError::InvalidMessage),
        };

        Ok(Self {
            id,
            payload: payload.to_vec(),
        })
    }

    /// Converts a `Message` to a byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        let len = self.payload.len() + 1;
        let len_bytes: [u8; 4] = (len as u32).to_be_bytes();
        let mut bytes = vec![0; 4 + len];
        bytes[0..4].copy_from_slice(&len_bytes);
        bytes[4] = self.id.clone() as u8;
        bytes[5..].copy_from_slice(&self.payload);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_unchoke_from_bytes() {
        let msg_type = 1u8.to_be_bytes();
        let payload = vec![];
        let msg = Message::from_bytes(msg_type, &payload).unwrap();

        assert_eq!(msg.id, MessageId::Unchoke);
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_message_interested_from_bytes() {
        let msg_type = 2u8.to_be_bytes();
        let payload = vec![];
        let msg = Message::from_bytes(msg_type, &payload).unwrap();

        assert_eq!(msg.id, MessageId::Interested);
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_message_request_to_bytes() {
        let index = 0u32.to_be_bytes();
        let begin = 0u32.to_be_bytes();
        let length = 16384u32.to_be_bytes();
        let payload = [index, begin, length].concat();
        let msg = Message::new(MessageId::Request, payload.clone());

        let bytes = msg.to_bytes();

        let len = 13u32.to_be_bytes();
        let msg_type = 6u8.to_be_bytes();
        let mut expected = vec![];
        expected.extend(&len);
        expected.extend(&msg_type);
        expected.extend(&payload);

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_message_interested_to_bytes() {
        let msg = Message::new(MessageId::Interested, vec![]);

        let bytes = msg.to_bytes();

        let len = 1u32.to_be_bytes();
        let msg_type = 2u8.to_be_bytes();
        let mut expected = vec![];
        expected.extend(&len);
        expected.extend(&msg_type);

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_request_to_bytes() {
        let index = 0u32;
        let begin = 0u32;
        let length = 16384u32;
        let request = Request::new(index, begin, length);

        let bytes = request.to_bytes();

        let mut expected = vec![];
        expected.extend(&index.to_be_bytes());
        expected.extend(&begin.to_be_bytes());
        expected.extend(&length.to_be_bytes());

        assert_eq!(bytes, expected);
    }

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
