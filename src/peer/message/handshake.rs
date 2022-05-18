/// Represents a handshake message
#[derive(Debug)]
pub struct Handshake {
    pstrlen: u8,
    pstr: String,
    reserved: [u8; 8],
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl Handshake {
    pub fn new(info_hash: Vec<u8>, peer_id: Vec<u8>) -> Self {
        Self {
            pstrlen: 19,
            pstr: "BitTorrent protocol".to_string(),
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.pstrlen];
        bytes.extend(self.pstr.as_bytes());
        bytes.extend(&self.reserved);
        bytes.extend(&self.info_hash);
        bytes.extend(&self.peer_id);
        bytes
    }
}
