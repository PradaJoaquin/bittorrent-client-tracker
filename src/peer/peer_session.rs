use std::fmt::Write;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use std::{
    io::{Read, Write as IOWrite},
    net::TcpStream,
};

use chrono::{DateTime, Local};
use sha1::{Digest, Sha1};

use crate::config::cfg::Cfg;
use crate::logger::logger_sender::LoggerSender;
use crate::torrent_handler::status::{AtomicTorrentStatus, AtomicTorrentStatusError};
use crate::torrent_parser::torrent::Torrent;
use crate::tracker::http::constants::PEER_ID;

use super::bt_peer::BtPeer;
use super::handshake::Handshake;
use super::peer_message::{Bitfield, Message, MessageError, MessageId, Request};
use super::session_status::SessionStatus;

const BLOCK_SIZE: u32 = 16384;

#[derive(Debug)]
pub enum PeerSessionError {
    HandshakeError,
    MessageError(MessageId),
    ErrorReadingMessage(io::Error),
    MessageDoesNotExist(MessageError),
    CouldNotConnectToPeer,
    ErrorDisconnectingFromPeer(AtomicTorrentStatusError),
    ErrorAbortingPiece(AtomicTorrentStatusError),
    ErrorSelectingPiece(AtomicTorrentStatusError),
    ErrorGettingCurrentDownloadingPieces(AtomicTorrentStatusError),
    ErrorGettingRemainingPieces(AtomicTorrentStatusError),
    ErrorNotifyingPieceDownloaded(AtomicTorrentStatusError),
    ErrorGettingPiece(AtomicTorrentStatusError),
    PieceHashDoesNotMatch,
    NoPiecesLeftToDownloadInThisPeer,
    ErrorGettingSessionsStatus(AtomicTorrentStatusError),
    PeerNotInterested,
    ErrorGettingBitfield(AtomicTorrentStatusError),
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
    current_piece: u32,
    config: Cfg,
    logger_sender: LoggerSender,
}

impl PeerSession {
    pub fn new(
        peer: BtPeer,
        torrent: Torrent,
        torrent_status: Arc<AtomicTorrentStatus>,
        config: Cfg,
        logger_sender: LoggerSender,
    ) -> PeerSession {
        PeerSession {
            torrent,
            peer,
            bitfield: Bitfield::new(vec![]),
            status: SessionStatus::default(),
            piece: vec![],
            torrent_status,
            current_piece: 0,
            config,
            logger_sender,
        }
    }

    /// Handshakes with an incoming leecher.
    pub fn handshake_incoming_leecher(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        self.set_stream_timeouts(stream)?;

        self.receive_handshake(stream)?;
        self.send_handshake(stream)?;
        self.logger_sender.info("Handshake successful");

        self.send_bitfield(stream)?;
        self.logger_sender.info("Bitfield sent");

        Ok(())
    }

    /// Sends an unchoke message to the peer to start sending pieces.
    pub fn unchoke_incoming_leecher(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        let mut id = self.read_message_from_stream(stream)?;
        while id != MessageId::Interested {
            // if we receive a `not interested` message, we close the connection.
            if id == MessageId::NotInterested {
                // peer disconnected
                return Err(PeerSessionError::PeerNotInterested);
            }
            // wait for the peer to send an interested message
            id = self.read_message_from_stream(stream)?;
        }
        self.send_unchoked(stream)?;

        loop {
            // TODO: Handle max connections.
            self.read_message_from_stream(stream)?;
        }
    }

    /// Starts a connection to an outgoing seeder to start downloading pieces.
    ///
    /// It returns an error if:
    /// - The connection could not be established
    /// - The handshake was not successful
    pub fn start_outgoing_seeder(&mut self) -> Result<(), PeerSessionError> {
        match self.start_outgoing_seeder_wrap() {
            Ok(_) => Ok(()),
            Err(e) => {
                self.torrent_status
                    .peer_disconnected(&self.peer)
                    .map_err(PeerSessionError::ErrorDisconnectingFromPeer)?;
                Err(e)
            }
        }
    }

    fn start_outgoing_seeder_wrap(&mut self) -> Result<(), PeerSessionError> {
        let peer_socket = format!("{}:{}", self.peer.ip, self.peer.port);

        let mut stream = TcpStream::connect(&peer_socket)
            .map_err(|_| PeerSessionError::CouldNotConnectToPeer)?;

        // set timeouts
        self.set_stream_timeouts(&mut stream)?;

        self.send_handshake(&mut stream)?;
        self.receive_handshake(&mut stream)?;

        self.logger_sender.info("Handshake successful");

        loop {
            self.read_message_from_stream(&mut stream)?;

            if self.status.choked && !self.status.interested {
                self.send_interested(&mut stream)?;
            }

            if !self.status.choked && self.status.interested {
                self.request_pieces(&mut stream)?;
            }
        }
    }

    fn set_stream_timeouts(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        stream
            .set_read_timeout(Some(Duration::from_secs(
                self.config.read_write_seconds_timeout,
            )))
            .map_err(|_| PeerSessionError::HandshakeError)?;

        stream
            .set_write_timeout(Some(Duration::from_secs(
                self.config.read_write_seconds_timeout,
            )))
            .map_err(|_| PeerSessionError::HandshakeError)?;

        Ok(())
    }

    fn request_pieces(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        loop {
            let piece_index = self
                .torrent_status
                .select_piece(&self.bitfield)
                .map_err(PeerSessionError::ErrorSelectingPiece)?;

            match piece_index {
                Some(piece_index) => {
                    self.current_piece = piece_index;
                    match self.download_piece(stream, piece_index) {
                        Ok(_) => {}
                        Err(e) => {
                            self.torrent_status
                                .piece_aborted(piece_index)
                                .map_err(PeerSessionError::ErrorAbortingPiece)?;

                            return Err(e);
                        }
                    }
                }
                None => {
                    return Err(PeerSessionError::NoPiecesLeftToDownloadInThisPeer);
                }
            };
        }
    }

    /// Downloads a piece from the peer given the piece index.
    fn download_piece(
        &mut self,
        stream: &mut TcpStream,
        piece_index: u32,
    ) -> Result<Vec<u8>, PeerSessionError> {
        self.piece = vec![]; // reset piece

        let entire_blocks_in_piece = self.download_with_pipeline(piece_index, stream)?;

        self.check_last_piece_block(piece_index, entire_blocks_in_piece, stream)?;

        self.validate_piece(&self.piece, piece_index)?;
        self.logger_sender
            .info(&format!("Piece {} downloaded!", piece_index));

        let remaining_pieces = self.torrent_status.downloaded_pieces();
        println!(
            "*** Torrent: {} - Pieces downloaded: {} / {}",
            self.torrent.name(),
            remaining_pieces,
            self.torrent.total_pieces()
        );

        Ok(self.piece.clone())
    }

    /// Downloads a piece in 'chunks' of blocks.
    ///
    /// If the pipelinening size in the config is 5, then it will request 5 blocks and wait for those 5 blocks to be received.
    ///
    /// If there are less than 5 blocks left in the piece, it will request the remaining blocks and wait for those blocks to be received.
    fn download_with_pipeline(
        &mut self,
        piece_index: u32,
        stream: &mut TcpStream,
    ) -> Result<u32, PeerSessionError> {
        let entire_blocks_in_piece = self.complete_blocks_in_torrent_piece(piece_index);
        let mut blocks_downloaded = 0;
        while blocks_downloaded < entire_blocks_in_piece {
            let blocks_to_download = if (entire_blocks_in_piece - blocks_downloaded)
                % self.config.pipelining_size
                == 0
            {
                self.config.pipelining_size
            } else {
                entire_blocks_in_piece - blocks_downloaded
            };

            let download_start_time = Local::now();

            // request blocks
            for block in 0..blocks_to_download {
                self.request_piece(
                    piece_index,
                    (block + blocks_downloaded) * BLOCK_SIZE,
                    BLOCK_SIZE,
                    stream,
                )?;
            }

            // Check that we receive a piece message.
            // If we receive another message we handle it accordingly.
            let mut current_blocks_downloaded = 0;
            while current_blocks_downloaded < blocks_to_download {
                if self.read_message_from_stream(stream)? == MessageId::Piece {
                    current_blocks_downloaded += 1;
                    blocks_downloaded += 1;
                }
            }
            // Calculate download speed
            let download_speed = self.calculate_kilobits_per_second(
                download_start_time,
                (blocks_to_download * BLOCK_SIZE).into(),
            );
            self.status.download_speed = download_speed;
            self.update_peer_status()?;
        }
        Ok(entire_blocks_in_piece)
    }

    fn check_last_piece_block(
        &mut self,
        piece_index: u32,
        entire_blocks_in_piece: u32,
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        let last_block_size = self.torrent.last_piece_size() % BLOCK_SIZE;

        let last_piece_index = self.torrent.total_pieces() - 1;

        if last_block_size != 0 && piece_index == last_piece_index {
            self.request_piece(
                piece_index,
                entire_blocks_in_piece * BLOCK_SIZE,
                last_block_size,
                stream,
            )?;
            while self.read_message_from_stream(stream)? != MessageId::Piece {
                continue;
            }
        }
        Ok(())
    }

    fn complete_blocks_in_torrent_piece(&self, piece_index: u32) -> u32 {
        let last_piece_index = self.torrent.total_pieces() - 1;

        if piece_index != last_piece_index {
            self.torrent.piece_length() / BLOCK_SIZE
        } else {
            let last_piece_size = self.torrent.last_piece_size();

            // If the last piece is multiple of the piece length, then is the same as the other pieces.
            if last_piece_size == 0 {
                self.torrent.piece_length() / BLOCK_SIZE
            } else {
                (last_piece_size as f64 / BLOCK_SIZE as f64).floor() as u32
            }
        }
    }

    fn calculate_kilobits_per_second(&self, start_time: DateTime<Local>, size: u64) -> f64 {
        let elapsed_time = Local::now().signed_duration_since(start_time);
        let elapsed_time_in_seconds = elapsed_time.num_milliseconds() as f64 / 1000.0;
        (size as f64 / elapsed_time_in_seconds) * 8.0 / 1024.0
    }

    fn update_peer_status(&mut self) -> Result<(), PeerSessionError> {
        self.torrent_status
            .update_peer_session_status(&self.peer, &self.status)
            .map_err(PeerSessionError::ErrorGettingSessionsStatus)?;
        Ok(())
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

        stream
            .read_exact(&mut length)
            .map_err(PeerSessionError::ErrorReadingMessage)?;
        let len = u32::from_be_bytes(length);

        // TODO: solucionar el problema de que el peer puede mandar un mensaje de mas de 16393 bytes. Cuando esta mandando cualquiera.
        // Issue: https://github.com/taller-1-fiuba-rust/22C1-La-Deymoneta/issues/101
        // Ahora que en el server la iniciacion esta dentro del Ok() esta fallando en el handshake, mirar ahi tambien.
        if len > BLOCK_SIZE * 10 {
            // handshake err temporal
            return Err(PeerSessionError::HandshakeError);
        }

        if len == 0 {
            return Ok(MessageId::KeepAlive);
        }

        let mut payload = vec![0; (len) as usize];

        stream
            .read_exact(&mut payload)
            .map_err(PeerSessionError::ErrorReadingMessage)?;

        let message =
            Message::from_bytes(&payload).map_err(PeerSessionError::MessageDoesNotExist)?;
        let id = message.id.clone();

        self.handle_message(message, stream)?;
        Ok(id)
    }

    /// Sends a handshake to the peer.
    ///
    /// It returns an error if the handshake could not be sent or the handshake was not successful.
    fn send_handshake(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        let info_hash = self
            .torrent
            .get_info_hash_as_bytes()
            .map_err(|_| PeerSessionError::HandshakeError)?;

        let handshake = Handshake::new(info_hash, PEER_ID.as_bytes().to_vec());
        stream
            .write_all(&handshake.as_bytes())
            .map_err(|_| PeerSessionError::HandshakeError)?;
        Ok(())
    }

    /// Reads a handshake from the peer.
    fn receive_handshake(&mut self, stream: &mut TcpStream) -> Result<Handshake, PeerSessionError> {
        let mut buffer = [0; 68];
        stream
            .read_exact(&mut buffer)
            .map_err(|_| PeerSessionError::HandshakeError)?;

        let handshake =
            Handshake::from_bytes(&buffer).map_err(|_| PeerSessionError::HandshakeError)?;

        let torrent_info_hash = self
            .torrent
            .get_info_hash_as_bytes()
            .map_err(|_| PeerSessionError::HandshakeError)?;

        if handshake.info_hash != torrent_info_hash {
            return Err(PeerSessionError::HandshakeError);
        }
        Ok(handshake)
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
        let interested_msg = Message::new(MessageId::Interested, vec![]);
        stream
            .write_all(&interested_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Interested))?;

        self.status.interested = true;

        Ok(())
    }

    /// Sends a unchoked message to the peer.
    fn send_unchoked(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        let unchoked_msg = Message::new(MessageId::Unchoke, vec![]);
        stream
            .write_all(&unchoked_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Unchoke))?;
        Ok(())
    }

    fn send_bitfield(&mut self, stream: &mut TcpStream) -> Result<(), PeerSessionError> {
        let bitfield = self
            .torrent_status
            .get_bitfield()
            .map_err(PeerSessionError::ErrorGettingBitfield)?;

        let bitfield_msg = Message::new(MessageId::Bitfield, bitfield.bitfield);
        stream
            .write_all(&bitfield_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Bitfield))?;
        Ok(())
    }

    /// Sends a piece message to the peer.
    fn send_piece(
        &mut self,
        index: u32,
        begin: u32,
        block: &[u8],
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        let mut payload = vec![];
        payload.extend(index.to_be_bytes());
        payload.extend(begin.to_be_bytes());
        payload.extend(block);

        let piece_msg = Message::new(MessageId::Piece, payload);
        stream
            .write_all(&piece_msg.as_bytes())
            .map_err(|_| PeerSessionError::MessageError(MessageId::Piece))?;

        self.logger_sender
            .info(&format!("Sent piece: {} / Offset: {}", index, begin));

        Ok(())
    }

    /// Handles a message received from the peer.
    fn handle_message(
        &mut self,
        message: Message,
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        match message.id {
            MessageId::Unchoke => self.handle_unchoke(),
            MessageId::Bitfield => self.handle_bitfield(message),
            MessageId::Piece => self.handle_piece(message),
            MessageId::Request => self.handle_request(message, stream)?,
            _ => {} // TODO: handle other messages,
        }
        Ok(())
    }

    /// Handles an unchoke message received from the peer.
    fn handle_unchoke(&mut self) {
        self.status.choked = false;
    }

    /// Handles a bitfield message received from the peer.
    fn handle_bitfield(&mut self, message: Message) {
        let bitfield = message.payload;
        self.bitfield = Bitfield::new(bitfield);
    }

    /// Handles a piece message received from the peer.
    fn handle_piece(&mut self, message: Message) {
        let block = &message.payload[8..];

        self.piece.append(&mut block.to_vec());
    }

    /// Handles a request message received from the peer.
    fn handle_request(
        &mut self,
        message: Message,
        stream: &mut TcpStream,
    ) -> Result<(), PeerSessionError> {
        let mut index: [u8; 4] = [0; 4];
        let mut begin: [u8; 4] = [0; 4];
        let mut length: [u8; 4] = [0; 4];
        index.copy_from_slice(&message.payload[0..4]);
        begin.copy_from_slice(&message.payload[4..8]);
        length.copy_from_slice(&message.payload[8..12]);

        let index = u32::from_be_bytes(index);
        let begin = u32::from_be_bytes(begin);
        let length = u32::from_be_bytes(length);

        let offset = index * self.torrent.piece_length() + begin;

        let upload_start_time = Local::now();

        let block = self
            .torrent_status
            .get_piece(index, offset as u64, length as usize)
            .map_err(PeerSessionError::ErrorGettingPiece)?;

        self.send_piece(index, begin, &block, stream)?;

        // Calculate upload speed
        let upload_speed = self.calculate_kilobits_per_second(upload_start_time, (length).into());
        self.status.upload_speed = upload_speed;
        self.update_peer_status()?;
        Ok(())
    }

    /// Validates the downloaded piece.
    ///
    /// Checks the piece hash and compares it to the hash in the torrent file.
    fn validate_piece(&self, piece: &[u8], piece_index: u32) -> Result<(), PeerSessionError> {
        let start = (piece_index * 20) as usize;
        let end = start + 20;

        let real_hash = &self.torrent.info.pieces[start..end];
        let real_piece_hash = self.convert_to_hex_string(real_hash);

        let hash = Sha1::digest(piece);
        let res_piece_hash = self.convert_to_hex_string(hash.as_slice());

        if real_piece_hash == res_piece_hash {
            self.torrent_status
                .piece_downloaded(piece_index, self.piece.clone())
                .map_err(PeerSessionError::ErrorNotifyingPieceDownloaded)?;
            Ok(())
        } else {
            Err(PeerSessionError::PieceHashDoesNotMatch)
        }
    }

    /// Converts a byte array to a hex string.
    fn convert_to_hex_string(&self, bytes: &[u8]) -> String {
        let mut res = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            match write!(&mut res, "{:02x}", b) {
                Ok(()) => (),
                Err(_) => self
                    .logger_sender
                    .warn("Error converting bytes to hex string!"),
            }
        }
        res
    }
}
