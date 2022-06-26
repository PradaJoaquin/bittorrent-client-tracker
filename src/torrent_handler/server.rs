use crate::config::cfg::Cfg;
use crate::logger::logger_sender::LoggerSender;
use crate::peer::bt_peer::BtPeer;
use crate::peer::peer_session::{PeerSession, PeerSessionError};
use crate::torrent_handler::status::AtomicTorrentStatus;
use crate::torrent_parser::torrent::Torrent;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use super::status::AtomicTorrentStatusError;

/// Struct for handling the server side.
///
/// To create a new `BtServer`, use BtServer::new(torrent, config, logger_sender).
#[derive(Debug)]
pub struct BtServer {
    config: Cfg,
    torrent: Torrent,
    torrent_status: Arc<AtomicTorrentStatus>,
    logger_sender: LoggerSender,
}

/// Posible BtServer errors.
#[derive(Debug)]
pub enum BtServerError {
    TorrentStatusError(AtomicTorrentStatusError),
    OpeningListenerError(std::io::Error),
    HandleConnectionError(std::io::Error),
    PeerSessionError(PeerSessionError),
}

impl BtServer {
    /// Creates a new `BtServer` from a `Torrent`, a `Config`, a `Torrent Status` and a `Logger Sender`.
    pub fn new(
        torrent: Torrent,
        config: Cfg,
        torrent_status: Arc<AtomicTorrentStatus>,
        logger_sender: LoggerSender,
    ) -> Self {
        Self {
            config,
            torrent,
            torrent_status,
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

    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<(), BtServerError> {
        let addr = stream
            .peer_addr()
            .map_err(BtServerError::HandleConnectionError)?;

        let peer = BtPeer::new(addr.ip().to_string(), addr.port() as i64);

        let mut peer_session = PeerSession::new(
            peer.clone(),
            self.torrent.clone(),
            self.torrent_status.clone(),
            self.config.clone(),
            self.logger_sender.clone(),
        );

        match peer_session.handshake_incoming_leecher(&mut stream) {
            Ok(_) => {
                self.unchoke_peer(peer_session, peer, stream);
            }
            Err(err) => {
                self.logger_sender.warn(&format!("{:?}", err));
            }
        }

        // peer connected

        // TODO: Handle max connections.

        Ok(())
    }

    fn unchoke_peer(&mut self, mut peer_session: PeerSession, peer: BtPeer, mut stream: TcpStream) {
        let peer_name = format!("{}:{}", peer.ip, peer.port);

        let builder = thread::Builder::new().name(format!(
            "Torrent: {} / Peer: {}",
            self.torrent.info.name, peer_name
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
            Err(err) => self.logger_sender.error(&format!("{:?}", err)),
        }
    }
}
