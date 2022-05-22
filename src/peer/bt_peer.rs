use std::{
    io::{Read, Write},
    net::{AddrParseError, TcpStream},
    num::ParseIntError,
};

use sha1::{Digest, Sha1};

use crate::{
    encoder_decoder::bencode::Bencode,
    peer::message::{handshake::Handshake, message::Message},
    torrent_parser::torrent::Torrent,
};

use super::message::{handshake::FromHandshakeError, message::MessageId};

const PEER_ID: &str = "LA_DEYMONETA_PAPA!!!";

/// `BtPeer` struct containing individual BtPeer information.
///
/// To create a new `BtPeer` use the method builder `from()`.
#[derive(Debug)]
pub struct BtPeer {
    pub peer_id: Vec<u8>,
    pub ip: String,
    pub port: i64,
    pub piece: Vec<u8>,
}

struct Request {
    index: u32,
    begin: u32,
    length: u32,
}

impl Request {
    fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index,
            begin,
            length,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; 12];
        bytes[0..4].copy_from_slice(&self.index.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.begin.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.length.to_be_bytes());
        bytes
    }
}

const BLOCK_SIZE: u32 = 16384;

/// Posible `BtPeer` errors
#[derive(Debug)]
pub enum FromBtPeerError {
    InvalidPeerId,
    InvalidIp,
    InvalidPort,
    NotADict,
}

#[derive(Debug)]
pub enum BtPeerError {
    HandshakeError(ParseIntError),
    FromHandshakeError(FromHandshakeError),
    AddrParseError(AddrParseError),
}

impl BtPeer {
    /// Builds a new `BtPeer` from a bencoded peer from the tracker response peer list.
    ///
    ///
    /// It returns an `FromBtPeerError` if:
    /// - The peer ID is invalid.
    /// - The peer IP is invalid.
    /// - The peer Port is invalid.
    /// - The bencoded peer is not a Dict.
    pub fn from(bencode: Bencode) -> Result<BtPeer, FromBtPeerError> {
        let mut peer_id: Vec<u8> = Vec::new();
        let mut ip: String = String::new();
        let mut port: i64 = 0;

        let d = match bencode {
            Bencode::BDict(d) => d,
            _ => return Err(FromBtPeerError::NotADict),
        };

        for (k, v) in d.iter() {
            if k == b"peer id" {
                peer_id = Self::create_peer_id(v)?;
            } else if k == b"ip" {
                ip = Self::create_ip(v)?;
            } else if k == b"port" {
                port = Self::create_port(v)?;
            }
        }

        Ok(BtPeer {
            peer_id,
            ip,
            port,
            piece: vec![],
        })
    }

    fn create_peer_id(bencode: &Bencode) -> Result<Vec<u8>, FromBtPeerError> {
        let peer_id = match bencode {
            Bencode::BString(s) => s.clone(),
            _ => return Err(FromBtPeerError::InvalidPeerId),
        };

        Ok(peer_id)
    }

    fn create_ip(bencode: &Bencode) -> Result<String, FromBtPeerError> {
        let ip = match bencode {
            Bencode::BString(s) => s,
            _ => return Err(FromBtPeerError::InvalidIp),
        };

        let ip = match String::from_utf8(ip.to_vec()) {
            Ok(s) => s,
            Err(_) => return Err(FromBtPeerError::InvalidIp),
        };

        Ok(ip)
    }

    fn create_port(bencode: &Bencode) -> Result<i64, FromBtPeerError> {
        let port = match bencode {
            Bencode::BNumber(n) => *n,
            _ => return Err(FromBtPeerError::InvalidPort),
        };

        Ok(port)
    }

    pub fn handshake(&mut self, torrent: &Torrent) -> Result<Handshake, BtPeerError> {
        let peer_socket = format!("{}:{}", self.ip, self.port)
            .parse::<std::net::SocketAddr>()
            .unwrap();

        let info_hash = torrent.get_info_hash_as_bytes().unwrap();

        let handshake = Handshake::new(info_hash, PEER_ID.as_bytes().to_vec());

        let mut stream = TcpStream::connect(&peer_socket).unwrap();
        stream.write_all(&handshake.to_bytes()).unwrap();

        let mut buffer = [0; 68];
        match stream.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(err) => println!("Error reading from stream: {}", err),
        }
        let handshake = Handshake::from_bytes(&buffer).map_err(BtPeerError::FromHandshakeError)?;
        println!("Received handshake: {:?}", handshake);

        let mut length = [0; 4];
        let mut msg_type = [0; 1];

        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let requests_to_do = torrent.info.piece_length as u32 / BLOCK_SIZE;
        let mut count = 0;

        loop {
            stream.read_exact(&mut length).unwrap();
            let len = u32::from_be_bytes(length);
            println!("Received message length: {:?}", len);

            stream.read_exact(&mut msg_type).unwrap();
            println!("Message type: {:?}", msg_type);

            let mut payload = vec![0; (len - 1) as usize];
            if len > 1 {
                stream.read_exact(&mut payload).unwrap();
            }
            println!("Payload: {:?}", payload);
            println!();

            let message = Message::from_bytes(&msg_type, &payload).unwrap();
            self.handle_message(message, &stream);

            if count < requests_to_do {
                Self::request_piece(0, count * BLOCK_SIZE, BLOCK_SIZE, &stream);
            } else {
                println!("All pieces requested");
                println!("piece: {:?}", self.piece);

                let hash = Sha1::digest(&self.piece);

                println!("hash: {:?}", hash);
            }
            count += 1;
        }
    }

    fn handle_message(&mut self, message: Message, stream: &TcpStream) {
        match message.id {
            MessageId::Unchoke => {
                println!("Received unchoke");

                // Now we can start requesting pieces
                let index = 0;
                let begin = 0;
                let length = 16384;
                Self::request_piece(index, begin, length, stream);
            }

            MessageId::Bitfield => Self::handle_bitfield(message),
            MessageId::Piece => self.handle_piece(message),
            _ => unimplemented!(),
        }
    }

    fn request_piece(index: u32, begin: u32, length: u32, mut stream: &TcpStream) {
        let payload = Request::new(index, begin, length).to_bytes();

        let request_msg = Message::new(MessageId::Request, payload);
        stream.write_all(&request_msg.to_bytes()).unwrap();
    }

    fn handle_bitfield(message: Message) {
        let bitfield = message.payload;

        println!("Received bitfield: {:?}", bitfield);
    }

    fn handle_piece(&mut self, mut message: Message) {
        // println!("Received piece: {:?}", message.payload);
        self.piece.append(&mut message.payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_from_bt_peer() {
        let mut dict = BTreeMap::new();
        dict.insert(b"peer id".to_vec(), Bencode::BString(b"peer id".to_vec()));
        dict.insert(b"ip".to_vec(), Bencode::BString(b"127.0.0.1".to_vec()));
        dict.insert(b"port".to_vec(), Bencode::BNumber(6868));

        let bencode = Bencode::BDict(dict);

        let bt_peer = BtPeer::from(bencode).unwrap();

        assert_eq!(bt_peer.peer_id, b"peer id".to_vec());
        assert_eq!(bt_peer.ip, "127.0.0.1");
        assert_eq!(bt_peer.port, 6868);
    }
}
