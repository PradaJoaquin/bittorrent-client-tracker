use std::collections::BTreeMap;
use std::fmt::Write;

use sha1::{Digest, Sha1};

use crate::encoder_decoder::bencode::{Bencode, ToBencode};

use super::info::Info;

#[derive(Debug, Clone)]
pub struct Torrent {
    pub announce_url: String,
    pub info: Info,
    pub info_hash: String,
}

#[derive(Debug, PartialEq)]
pub enum FromTorrentError {
    MissingAnnounce,
    MissingInfo,
    InfoHashError,
    NotADict,
}

impl Torrent {
    pub fn from(bencode: Bencode) -> Result<Torrent, FromTorrentError> {
        let mut announce_url = String::new();
        let mut info: Option<Info> = None;

        let d = match bencode {
            Bencode::BDict(s) => s,
            _ => return Err(FromTorrentError::NotADict),
        };

        for (k, v) in d.iter() {
            if k == b"announce" {
                announce_url = Torrent::create_announce(v)?;
            } else if k == b"info" {
                info = Some(Torrent::create_info(v)?);
            }
        }

        if announce_url.is_empty() {
            return Err(FromTorrentError::MissingAnnounce);
        }

        let info = match info {
            Some(x) => x,
            None => return Err(FromTorrentError::MissingInfo),
        };

        let info_hash = Torrent::create_info_hash(&info)?;

        Ok(Torrent {
            announce_url,
            info,
            info_hash,
        })
    }

    fn create_announce(bencode: &Bencode) -> Result<String, FromTorrentError> {
        let announce_url = match bencode {
            Bencode::BString(s) => s,
            _ => return Err(FromTorrentError::MissingAnnounce),
        };

        let announce_url = match String::from_utf8(announce_url.to_vec()) {
            Ok(s) => s,
            Err(_) => return Err(FromTorrentError::MissingAnnounce),
        };

        Ok(announce_url)
    }

    fn create_info(bencode: &Bencode) -> Result<Info, FromTorrentError> {
        let info = match Info::from(bencode) {
            Ok(x) => x,
            Err(_) => return Err(FromTorrentError::MissingInfo),
        };

        Ok(info)
    }

    pub fn create_info_hash(info: &Info) -> Result<String, FromTorrentError> {
        let bencoded_info = Bencode::encode(info);
        let hash = Sha1::digest(bencoded_info);

        let mut hex_string = String::with_capacity(hash.len() * 2);

        for b in hash {
            match write!(&mut hex_string, "{:02x}", b) {
                Ok(_) => (),
                Err(_) => return Err(FromTorrentError::InfoHashError),
            }
        }

        Ok(hex_string)
    }
}

impl ToBencode for Torrent {
    fn to_bencode(&self) -> Bencode {
        let mut m = BTreeMap::new();
        m.insert(b"announce_url".to_vec(), self.announce_url.to_bencode());
        m.insert(b"info".to_vec(), self.info.to_bencode());
        Bencode::BDict(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_torrent_full() {
        let announce = String::from("http://example.com/announce");
        let info_len = 10;
        let info_name = String::from("example");
        let info_piece_len = 20;
        let info_pieces = String::from("test").into_bytes();

        let info_bencode = build_info_bencode(
            info_len,
            info_name.clone().into_bytes(),
            info_piece_len,
            info_pieces.clone(),
        );
        let torrent_bencode =
            build_torrent_bencode(announce.clone().into_bytes(), info_bencode.clone());

        let info = Info::from(&Bencode::BDict(info_bencode)).unwrap();
        let info_hash = Torrent::create_info_hash(&info).unwrap();

        let torrent = Torrent::from(torrent_bencode).unwrap();

        assert_eq!(torrent.announce_url, announce);
        assert_eq!(torrent.info.length, info_len);
        assert_eq!(torrent.info.name, info_name);
        assert_eq!(torrent.info.piece_length, info_piece_len);
        assert_eq!(torrent.info.pieces, info_pieces);
        assert_eq!(torrent.info_hash, info_hash);
    }

    #[test]
    fn test_from_torrent_empty() {
        let torrent_bencode = Bencode::BDict(BTreeMap::new());

        let actual_err = Torrent::from(torrent_bencode).unwrap_err();
        let expected_err = FromTorrentError::MissingAnnounce;

        assert_eq!(actual_err, expected_err);
    }

    #[test]
    fn test_from_torrent_missing_announce() {
        let mut m = BTreeMap::new();
        m.insert(b"info".to_vec(), Bencode::BDict(BTreeMap::new()));
        let torrent_bencode = Bencode::BDict(m);

        let actual_err = Torrent::from(torrent_bencode).unwrap_err();
        let expected_err = FromTorrentError::MissingAnnounce;

        assert_eq!(actual_err, expected_err);
    }

    #[test]
    fn test_from_torrent_missing_info() {
        let announce = String::from("http://example.com/announce").into_bytes();
        let mut m = BTreeMap::new();
        m.insert(b"announce".to_vec(), Bencode::BString(announce));
        let torrent_bencode = Bencode::BDict(m);

        let actual_err = Torrent::from(torrent_bencode).unwrap_err();
        let expected_err = FromTorrentError::MissingInfo;

        assert_eq!(actual_err, expected_err);
    }

    #[test]
    fn test_from_torrent_not_a_dict() {
        let torrent_bencode = Bencode::BString(String::from("test").into_bytes());

        let actual_err = Torrent::from(torrent_bencode).unwrap_err();
        let expected_err = FromTorrentError::NotADict;

        assert_eq!(actual_err, expected_err);
    }

    fn build_info_bencode(
        length: i64,
        name: Vec<u8>,
        pieces_len: i64,
        pieces: Vec<u8>,
    ) -> BTreeMap<Vec<u8>, Bencode> {
        let mut info = BTreeMap::new();
        info.insert(b"length".to_vec(), Bencode::BNumber(length));
        info.insert(b"name".to_vec(), Bencode::BString(name));
        info.insert(b"piece length".to_vec(), Bencode::BNumber(pieces_len));
        info.insert(b"pieces".to_vec(), Bencode::BString(pieces));

        info
    }

    fn build_torrent_bencode(announce: Vec<u8>, info: BTreeMap<Vec<u8>, Bencode>) -> Bencode {
        let mut dict = BTreeMap::new();

        dict.insert(b"announce".to_vec(), Bencode::BString(announce));
        dict.insert(b"info".to_vec(), Bencode::BDict(info));

        Bencode::BDict(dict)
    }
}
