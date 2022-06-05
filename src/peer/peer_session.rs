use std::fmt::Write;
use std::io;
use std::sync::Arc;
use std::{
    io::{Read, Write as IOWrite},
    net::TcpStream,
};

use sha1::{Digest, Sha1};

use crate::torrent_handler::status::AtomicTorrentStatus;
use crate::torrent_parser::torrent::Torrent;
use crate::tracker::http::constants::PEER_ID;

use super::bt_peer::BtPeer;
use super::handshake::Handshake;
use super::peer_message::{Bitfield, FromMessageError, Message, MessageId, Request};
use super::session_status::SessionStatus;

const BLOCK_SIZE: u32 = 16384;

#[derive(Debug)]
pub enum PeerSessionError {
    HandshakeError,
    MessageError(MessageId),
    ErrorReadingMessage(io::Error),
    FromMessageError(FromMessageError),
    CouldNotConnectToPeer,
}

/// A PeerSession represents a connection to a peer.
///
/// It is used to send and receive messages from a peer.
pub struct PeerSession {
    torrent: Torrent,
    peer: BtPeer,
    bitfield: Bitfield,
    status: SessionStatus,
    piece: Vec<u8>,
    torrent_status: Arc<AtomicTorrentStatus>,
}

impl PeerSession {
    pub fn new(
        peer: BtPeer,
        torrent: Torrent,
        torrent_status: Arc<AtomicTorrentStatus>,
    ) -> PeerSession {
        PeerSession {
            torrent,
            peer,
            bitfield: Bitfield::new(vec![]),
            status: SessionStatus::default(),
            piece: vec![],
            torrent_status,
        }
    }

    /// Starts a connection to the peer.
    ///
    /// It returns an error if:
    /// - The connection could not be established
    /// - The handshake was not successful
    pub fn start(&mut self) -> Result<(), PeerSessionError> {
        let peer_socket = format!("{}:{}", self.peer.ip, self.peer.port);

        let mut stream = TcpStream::connect(&peer_socket)
            .map_err(|_| PeerSessionError::CouldNotConnectToPeer)?;

        let handshake = self.send_handshake(&mut stream)?;
        println!("Received handshake: {:?}", handshake);

        loop {
            self.read_message_from_stream(&mut stream)?;

            if self.status.choked && !self.status.interested {
                self.send_interested(&mut stream)?;
            }

            if !self.status.choked && self.status.interested {
                println!("Requesting pieces...");
                loop {
                    match self.torrent_status.select_piece(&self.bitfield).unwrap() {
                        Some(piece_index) => {
                            println!("\n\n********* Downloading piece {}...", piece_index);
                            self.download_piece(&mut stream, piece_index)?
                        }
                        None => {
                            println!("No pieces left to download in this peer");
                            self.torrent_status.peer_diconnected().unwrap();
                            break;
                        }
                    };
                }
            }
        }
    }

    /// Downloads a piece from the peer given the piece index.
    fn download_piece(
        &mut self,
        stream: &mut TcpStream,
        piece_index: u32,
    ) -> Result<Vec<u8>, PeerSessionError> {
        self.piece = vec![]; // reset piece

        if piece_index == 1006 {
            self.request_piece(
                piece_index,
                0,
                (self.torrent.info.length % self.torrent.info.piece_length) as u32,
                stream,
            )?;
            self.read_message_from_stream(stream).unwrap();
        } else {
            let total_blocks_in_piece = self.torrent.info.piece_length as u32 / BLOCK_SIZE;

            for block in 0..total_blocks_in_piece {
                self.request_piece(piece_index, block * BLOCK_SIZE, BLOCK_SIZE, stream)?;
                while self.read_message_from_stream(stream)? != MessageId::Piece {
                    continue;
                }
            }
        }

        self.validate_piece(&self.piece, piece_index);
        println!("Piece {} downloaded!", piece_index);

        println!(
            "********** PIECES REMAINING: {}",
            self.torrent_status.remaining_pieces().unwrap()
        );
        Ok(self.piece.clone())
    }

    /// Reads & handles a message from the stream.
    ///
    /// It returns an error if:
    /// - The message could not be read
    fn read_message_from_stream(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<MessageId, PeerSessionError> {
        let mut length = [0; 4];
        let mut msg_type = [0; 1];

        stream
            .read_exact(&mut length)
            .map_err(PeerSessionError::ErrorReadingMessage)?;
        let len = u32::from_be_bytes(length);

        stream
            .read_exact(&mut msg_type)
            .map_err(PeerSessionError::ErrorReadingMessage)?;

        let mut payload = vec![0; (len - 1) as usize];
        if len > 1 {
            stream
                .read_exact(&mut payload)
                .map_err(PeerSessionError::ErrorReadingMessage)?;
        }

        let message =
            Message::from_bytes(msg_type, &payload).map_err(PeerSessionError::FromMessageError)?;
        let id = message.id.clone();

        self.handle_message(message);
        Ok(id)
    }

    /// Sends a handshake to the peer and returns the handshake received from the peer.
    ///
    /// It returns an error if the handshake could not be sent or the handshake was not successful.
    fn send_handshake(&mut self, stream: &mut TcpStream) -> Result<Handshake, PeerSessionError> {
        let info_hash = self
            .torrent
            .get_info_hash_as_bytes()
            .map_err(|_| PeerSessionError::HandshakeError)?;

        let handshake = Handshake::new(info_hash, PEER_ID.as_bytes().to_vec());
        stream
            .write_all(&handshake.as_bytes())
            .map_err(|_| PeerSessionError::HandshakeError)?;

        let mut buffer = [0; 68];
        stream
            .read_exact(&mut buffer)
            .map_err(|_| PeerSessionError::HandshakeError)?;

        Handshake::from_bytes(&buffer).map_err(|_| PeerSessionError::HandshakeError)
    }

    /// Sends a request message to the peer.
    fn request_piece(
        &self,
        index: u32,
        begin: u32,
        length: u32,
        mut stream: &TcpStream,
    ) -> Result<(), PeerSessionError> {
        let payload = Request::new(index, begin, length).as_bytes();

        let request_msg = Message::new(MessageId::Request, payload);
        stream
            .write_all(&request_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Request))?;
        Ok(())
    }

    /// Sends an interested message to the peer.
    fn send_interested(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        println!("Sending interested...");

        let interested_msg = Message::new(MessageId::Interested, vec![]);
        stream
            .write_all(&interested_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Interested))?;

        self.status.interested = true;

        Ok(())
    }

    /// Handles a message received from the peer.
    fn handle_message(&mut self, message: Message) {
        match message.id {
            MessageId::Unchoke => self.handle_unchoke(),
            MessageId::Bitfield => self.handle_bitfield(message),
            MessageId::Piece => self.handle_piece(message),
            _ => println!("Received message: {:?}", message),
        }
    }

    /// Handles an unchoke message received from the peer.
    fn handle_unchoke(&mut self) {
        println!("Received unchoke");
        self.status.choked = false;
    }

    /// Handles a bitfield message received from the peer.
    fn handle_bitfield(&mut self, message: Message) {
        println!("Received bitfield");
        let bitfield = message.payload;
        self.bitfield = Bitfield::new(bitfield);
    }

    /// Handles a piece message received from the peer.
    fn handle_piece(&mut self, message: Message) {
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
            self.torrent_status
                .piece_downloaded(piece_index, self.piece.clone())
                .unwrap();
        } else {
            println!("Piece {} hash does not match!", piece_index);
        }
    }

    /// Converts a byte array to a hex string.
    fn convert_to_hex_string(&self, bytes: &[u8]) -> String {
        let mut res = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            match write!(&mut res, "{:02x}", b) {
                Ok(()) => (),
                Err(_) => println!("Error converting bytes to hex string!"),
            }
        }
        res
    }
}
