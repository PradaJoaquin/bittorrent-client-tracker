use crate::encoder_decoder::bencode::Bencode;

/// `BtPeer` struct containing individual BtPeer information.
///
/// To create a new `BtPeer` use the method builder `from()`.
#[derive(Debug)]
pub struct BtPeer {
    pub peer_id: Option<Vec<u8>>,
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

impl BtPeer {
    /// Builds a new `BtPeer` decoding a bencoded Vec<u8> cointaining the BtPeer information.
    pub fn new(ip: String, port: i64) -> Self {
        Self {
            peer_id: None,
            ip,
            port,
        }
    }

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
            peer_id: Some(peer_id),
            ip,
            port,
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

        assert_eq!(bt_peer.peer_id, Some(b"peer id".to_vec()));
        assert_eq!(bt_peer.ip, "127.0.0.1");
        assert_eq!(bt_peer.port, 6868);
    }

    #[test]
    fn test_new_peer() {
        let bt_peer = BtPeer::new("127.0.0.1".to_string(), 6868);

        assert_eq!(bt_peer.peer_id, None);
        assert_eq!(bt_peer.ip, "127.0.0.1");
        assert_eq!(bt_peer.port, 6868);
    }
}
