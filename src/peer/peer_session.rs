use std::fmt::Write;
use std::{
    io::{Read, Write as IOWrite},
    net::TcpStream,
};

use sha1::{Digest, Sha1};

use crate::{
    peer::message::{handshake::Handshake, message::Message},
    torrent_parser::torrent::Torrent,
};

use super::bt_peer::BtPeer;
use super::message::handshake::FromHandshakeError;
use super::message::message::{Bitfield, MessageId, Request};

const PEER_ID: &str = "LA_DEYMONETA_PAPA!!!";
const BLOCK_SIZE: u32 = 16384;

#[derive(Debug)]
pub enum PeerSessionError {
    HandshakeError(FromHandshakeError),
    MessageError(MessageId),
    RequestError(Request),
}

#[derive(Debug)]
pub struct PeerStatus {
    choked: bool,
    interested: bool,
}

impl PeerStatus {
    pub fn new() -> PeerStatus {
        PeerStatus {
            choked: true,
            interested: false,
        }
    }
}

impl Default for PeerStatus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PeerSession {
    torrent: Torrent,
    peer: BtPeer,
    bitfield: Bitfield,
    status: PeerStatus,
    piece: Vec<u8>,
}

impl PeerSession {
    pub fn new(peer: BtPeer, torrent: Torrent) -> PeerSession {
        PeerSession {
            torrent,
            peer,
            bitfield: Bitfield::new(vec![]),
            status: PeerStatus::new(),
            piece: vec![],
        }
    }

    pub fn start(&mut self) -> Result<(), PeerSessionError> {
        let peer_socket = format!("{}:{}", self.peer.ip, self.peer.port)
            .parse::<std::net::SocketAddr>()
            .unwrap();

        let mut stream = TcpStream::connect(&peer_socket).unwrap();

        let handshake = self.send_handshake(&mut stream)?;
        println!("Received handshake: {:?}", handshake);

        let piece_index = 0;
        self.download_piece(stream, piece_index);

        Ok(())
    }

    fn send_handshake(&mut self, stream: &mut TcpStream) -> Result<Handshake, PeerSessionError> {
        let info_hash = self.torrent.get_info_hash_as_bytes().unwrap();
        let handshake = Handshake::new(info_hash, PEER_ID.as_bytes().to_vec());
        stream.write_all(&handshake.to_bytes()).unwrap();

        let mut buffer = [0; 68];
        match stream.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(err) => println!("Error reading from stream: {}", err),
        }

        Handshake::from_bytes(&buffer).map_err(PeerSessionError::HandshakeError)
    }

    fn download_piece(&mut self, mut stream: TcpStream, piece_index: u32) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .unwrap();

        let mut length = [0; 4];
        let mut msg_type = [0; 1];

        let requests_to_do = self.torrent.info.piece_length as u32 / BLOCK_SIZE;
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
            println!();

            let message = Message::from_bytes(&msg_type, &payload).unwrap();
            self.handle_message(message);

            let has_piece = self.bitfield.has_piece(piece_index);
            println!("Has piece: {:?}", has_piece);
            println!("Requests to do: {:?}", requests_to_do);

            if !self.status.interested {
                self.send_interested(&mut stream);
            }

            if !self.status.choked && count < requests_to_do {
                if has_piece {
                    println!("Sending request: {:?}", count);
                    self.request_piece(piece_index, count * BLOCK_SIZE, BLOCK_SIZE, &stream);
                    count += 1;
                }
            } else if count >= requests_to_do {
                println!("Piece {} downloaded!", piece_index);
                self.validate_piece(&self.piece, piece_index);
            }
        }
    }

    fn handle_message(&mut self, message: Message) {
        match message.id {
            MessageId::Unchoke => self.handle_unchoke(),
            MessageId::Bitfield => self.handle_bitfield(message),
            MessageId::Piece => self.handle_piece(message),
            _ => todo!(),
        }
    }

    fn request_piece(&self, index: u32, begin: u32, length: u32, mut stream: &TcpStream) {
        let payload = Request::new(index, begin, length).to_bytes();

        let request_msg = Message::new(MessageId::Request, payload);
        stream.write_all(&request_msg.to_bytes()).unwrap();
    }

    fn send_interested(&mut self, stream: &mut TcpStream) {
        println!("Sending interested...");
        let interested_msg = Message::new(MessageId::Interested, vec![]);
        stream.write_all(&interested_msg.to_bytes()).unwrap();
        self.status.interested = true;
    }

    fn handle_unchoke(&mut self) {
        println!("Received unchoke");
        self.status.choked = false;
    }

    fn handle_bitfield(&mut self, message: Message) {
        println!("Received bitfield");
        let bitfield = message.payload;
        self.bitfield = Bitfield::new(bitfield);
    }

    fn handle_piece(&mut self, message: Message) {
        println!("Received piece");
        let index = &message.payload[0..4];
        let begin = &message.payload[4..8];
        let block = &message.payload[8..];

        self.piece.append(&mut block.to_vec());
    }

    fn validate_piece(&self, piece: &[u8], piece_index: u32) {
        let start = (piece_index * 20) as usize;
        let end = start + 20;

        let real_hash = &self.torrent.info.pieces[start..end];
        let real_piece_hash = self.convert_to_hex_string(real_hash);

        let hash = Sha1::digest(piece);
        let res_piece_hash = self.convert_to_hex_string(hash.as_slice());

        println!("Real piece hash: {:?}", real_piece_hash);
        println!("Downloaded piece hash: {:?}", res_piece_hash);

        if real_piece_hash == res_piece_hash {
            println!("Piece {} hash matches!", piece_index);
        } else {
            println!("Piece {} hash does not match!", piece_index);
        }
    }

    fn convert_to_hex_string(&self, bytes: &[u8]) -> String {
        let mut res = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            write!(&mut res, "{:02x}", b).unwrap();
        }
        res
    }
}
