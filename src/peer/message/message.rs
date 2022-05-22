#[derive(Debug, Clone)]
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
    pub fn new(id: MessageId, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }

    pub fn from_bytes(msg_type: &[u8], payload: &[u8]) -> Result<Self, FromMessageError> {
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let len = self.payload.len() + 1;
        println!("*** message len: {}", len);
        let len_bytes: [u8; 4] = (len as u32).to_be_bytes();
        let mut bytes = vec![0; 4 + len];
        bytes[0..4].copy_from_slice(&len_bytes);
        bytes[4] = self.id.clone() as u8;
        bytes[5..].copy_from_slice(&self.payload);
        bytes
    }
}
