use crate::config::cfg::Cfg;
use crate::logger::logger_sender::LoggerSender;
use crate::peer::bt_peer::{BtPeer, BtPeerError};
use crate::peer::peer_session::{PeerSession, PeerSessionError};
use crate::torrent_handler::status::{AtomicTorrentStatus, AtomicTorrentStatusError};
use crate::torrent_parser::torrent::Torrent;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Struct for handling the server side.
///
/// To create a new `BtServer`, use BtServer::new(torrent, config, logger_sender).
#[derive(Debug)]
pub struct BtServer {
    config: Cfg,
    torrents_with_status: HashMap<Torrent, Arc<AtomicTorrentStatus>>,
    logger_sender: LoggerSender,
}

/// Posible BtServer errors.
#[derive(Debug)]
pub enum BtServerError {
    TorrentStatusError(AtomicTorrentStatusError),
    OpeningListenerError(std::io::Error),
    HandleConnectionError(std::io::Error),
    PeerSessionError(PeerSessionError),
    BtPeerError(BtPeerError),
    TorrentNotFound(Vec<u8>),
    ErrorSettingStreamTimeout,
}

impl BtServer {
    /// Creates a new `BtServer` from a `HashMap` containing a torrent with its `AtomicTorrentStatus`, a `Config` and a `Logger Sender`.
    pub fn new(
        torrents_with_status: HashMap<Torrent, Arc<AtomicTorrentStatus>>,
        config: Cfg,
        logger_sender: LoggerSender,
    ) -> Self {
        Self {
            config,
            torrents_with_status,
            logger_sender,
        }
    }

    /// Starts the server and starts listening for connections.
    ///
    /// # Errors
    /// - `OpeningListenerError` if the TcpLister couldn't be opened.
    pub fn init(&mut self) -> Result<(), BtServerError> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.tcp_port))
            .map_err(BtServerError::OpeningListenerError)?;

        self.logger_sender
            .info("Server started, listening for connections.");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => match self.handle_connection(stream) {
                    Ok(_) => (),
                    Err(e) => self
                        .logger_sender
                        .warn(&format!("Could't handle incoming connection: {:?}", e)),
                },
                Err(e) => self
                    .logger_sender
                    .warn(&format!("Could't handle incoming connection: {:?}", e)),
            }
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), BtServerError> {
        let addr = stream
            .peer_addr()
            .map_err(BtServerError::HandleConnectionError)?;

        // set timeouts
        self.set_stream_timeouts(&mut stream)?;

        let mut peer = BtPeer::new(addr.ip().to_string(), addr.port() as i64);

        let info_hash = peer.receive_handshake(&mut stream).map_err(|err| {
            self.logger_sender.warn(&format!(
                "{:?} for peer: {}:{}",
                err,
                addr.ip(),
                addr.port() as i64
            ));
            BtServerError::BtPeerError(err)
        })?;

        // See if the torrent is in the list of torrents.
        let (torrent, torrent_status) =
            match self.torrents_with_status.iter().find(|(torrent, _)| {
                match torrent.get_info_hash_as_bytes() {
                    Ok(info_hash_bytes) => info_hash_bytes == info_hash,
                    Err(_) => false,
                }
            }) {
                Some((torrent, torrent_status)) => (torrent, torrent_status),
                None => return Err(BtServerError::TorrentNotFound(info_hash)),
            };

        let mut peer_session = PeerSession::new(
            peer.clone(),
            torrent.clone(),
            torrent_status.clone(),
            self.config.clone(),
            self.logger_sender.clone(),
        )
        .map_err(BtServerError::PeerSessionError)?;

        match peer_session.handshake_incoming_leecher(&mut stream) {
            Ok(_) => {
                self.unchoke_peer(peer_session, peer, stream, torrent.clone(), torrent_status)?;
            }
            Err(err) => {
                self.logger_sender.warn(&format!("{:?}", err));
            }
        }

        // peer connected

        // TODO: Handle max connections.

        Ok(())
    }

    /// Sets read and write timeouts for the stream.
    fn set_stream_timeouts(&self, stream: &mut TcpStream) -> Result<(), BtServerError> {
        stream
            .set_read_timeout(Some(Duration::from_secs(
                self.config.read_write_seconds_timeout,
            )))
            .map_err(|_| BtServerError::ErrorSettingStreamTimeout)?;

        stream
            .set_write_timeout(Some(Duration::from_secs(
                self.config.read_write_seconds_timeout,
            )))
            .map_err(|_| BtServerError::ErrorSettingStreamTimeout)?;
        Ok(())
    }

    fn unchoke_peer(
        &self,
        mut peer_session: PeerSession,
        peer: BtPeer,
        mut stream: TcpStream,
        torrent: Torrent,
        torrent_status: &Arc<AtomicTorrentStatus>,
    ) -> Result<(), BtServerError> {
        torrent_status
            .peer_connected(&peer)
            .map_err(BtServerError::TorrentStatusError)?;
        let peer_name = format!("{}:{}", peer.ip, peer.port);

        let builder = thread::Builder::new().name(format!(
            "Torrent: {} / Peer: {}",
            torrent.info.name, peer_name
        ));
        let peer_logger_sender = self.logger_sender.clone();

        let join =
            builder.spawn(
                move || match peer_session.unchoke_incoming_leecher(&mut stream) {
                    Ok(_) => (),
                    Err(err) => {
                        peer_logger_sender.warn(&format!("{:?}", err));
                    }
                },
            );
        match join {
            Ok(_) => (),
            Err(err) => {
                self.logger_sender.error(&format!("{:?}", err));
                torrent_status
                    .peer_disconnected(&peer)
                    .map_err(BtServerError::TorrentStatusError)?;
            }
        }
        Ok(())
    }
}
