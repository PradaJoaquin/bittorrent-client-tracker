use std::{
    io::{Read, Write},
    net::{AddrParseError, TcpStream},
    num::ParseIntError,
};

use crate::{
    encoder_decoder::bencode::Bencode, peer::message::handshake::Handshake,
    torrent_parser::torrent::Torrent,
};

use super::message::handshake::FromHandshakeError;

const PEER_ID: &str = "LA_DEYMONETA_PAPA!!!";

/// `BtPeer` struct containing individual BtPeer information.
///
/// To create a new `BtPeer` use the method builder `from()`.
#[derive(Debug)]
pub struct BtPeer {
    pub peer_id: Vec<u8>,
    pub ip: String,
    pub port: i64,
}

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

        Ok(BtPeer { peer_id, ip, port })
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

    pub fn handshake(&self, torrent: &Torrent) -> Result<Handshake, BtPeerError> {
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

        Handshake::from_bytes(&buffer).map_err(BtPeerError::FromHandshakeError)
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
