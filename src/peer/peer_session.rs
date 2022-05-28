use std::fmt::Write;
use std::{
    io::{Read, Write as IOWrite},
    net::TcpStream,
};

use sha1::{Digest, Sha1};

use crate::torrent_parser::torrent::Torrent;

use super::bt_peer::BtPeer;
use super::message::handshake::{FromHandshakeError, Handshake};
use super::message::message::{Bitfield, Message, MessageId, Request};
use super::peer_status::PeerStatus;

const PEER_ID: &str = "LA_DEYMONETA_PAPA!!!";
const BLOCK_SIZE: u32 = 16384;

#[derive(Debug)]
pub enum PeerSessionError {
    HandshakeError(FromHandshakeError),
    MessageError(MessageId),
    RequestError(Request),
}

/// A PeerSession represents a connection to a peer.
///
/// It is used to send and receive messages from a peer.
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
            status: PeerStatus::default(),
            piece: vec![],
        }
    }

    /// Starts a connection to the peer.
    ///
    /// It returns an error if:
    /// - The connection could not be established
    /// - The handshake was not successful
    pub fn start(&mut self) -> Result<(), PeerSessionError> {
        let peer_socket = format!("{}:{}", self.peer.ip, self.peer.port)
            .parse::<std::net::SocketAddr>()
            .unwrap();

        let mut stream = TcpStream::connect(&peer_socket).unwrap();

        let handshake = self.send_handshake(&mut stream)?;
        println!("Received handshake: {:?}", handshake);

        let total_pieces = (self.torrent.info.length / self.torrent.info.piece_length) as u32;

        loop {
            self.read_message_from_stream(&mut stream);

            if self.status.choked && !self.status.interested {
                self.send_interested(&mut stream);
            }

            if !self.status.choked && self.status.interested {
                println!("Requesting pieces...");
                for piece_index in 0..total_pieces {
                    let has_piece = self.bitfield.has_piece(piece_index);
                    println!("Has piece {}: {:?}", piece_index, has_piece);

                    if has_piece {
                        println!("\n\n********* Downloading piece {}...", piece_index);
                        self.download_piece(&mut stream, piece_index);
                    }
                }
                println!("\n\n********* Download complete!");
                break;
            }
        }

        Ok(())
    }

    /// Downloads a piece from the peer given the piece index.
    fn download_piece(&mut self, stream: &mut TcpStream, piece_index: u32) {
        let total_blocks_in_piece = self.torrent.info.piece_length as u32 / BLOCK_SIZE;
        println!(
            "Total blocks for piece {}: {}",
            piece_index, total_blocks_in_piece
        );

        for block in 0..total_blocks_in_piece {
            println!("Sending request: {}/{}", block, total_blocks_in_piece);
            self.request_piece(piece_index, block * BLOCK_SIZE, BLOCK_SIZE, stream);
            self.read_message_from_stream(stream);
        }

        self.validate_piece(&self.piece, piece_index);
        println!("Piece {} downloaded!", piece_index);

        self.piece = vec![]; // reset piece
    }

    fn read_message_from_stream(&mut self, stream: &mut TcpStream) {
        let mut length = [0; 4];
        let mut msg_type = [0; 1];

        stream.read_exact(&mut length).unwrap();
        let len = u32::from_be_bytes(length);

        stream.read_exact(&mut msg_type).unwrap();

        let mut payload = vec![0; (len - 1) as usize];
        if len > 1 {
            stream.read_exact(&mut payload).unwrap();
        }
        println!();

        let message = Message::from_bytes(&msg_type, &payload).unwrap();
        self.handle_message(message);
    }

    /// Sends a handshake to the peer and returns the handshake received from the peer.
    ///
    /// It returns an error if the handshake could not be sent or the handshake was not successful.
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

    /// Validates the downloaded piece.
    ///
    /// Checks the piece hash and compares it to the hash in the torrent file.
    fn validate_piece(&self, piece: &[u8], piece_index: u32) {
        println!("\nValidating piece {}...", piece_index);
        let start = (piece_index * 20) as usize;
        let end = start + 20;

        let real_hash = &self.torrent.info.pieces[start..end];
        let real_piece_hash = self.convert_to_hex_string(real_hash);

        let hash = Sha1::digest(piece);
        let res_piece_hash = self.convert_to_hex_string(hash.as_slice());

        println!("Real piece hash: {:?}", real_piece_hash);
        println!("Downloaded piece hash: {:?}", res_piece_hash);

        if real_piece_hash == res_piece_hash {
            println!("Piece {} hash matches!\n\n", piece_index);
        } else {
            panic!("Piece {} hash does not match!", piece_index);
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
