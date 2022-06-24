use super::{
    server::{BtServer, BtServerError},
    status::{AtomicTorrentStatus, AtomicTorrentStatusError},
};
use crate::{
    config::cfg::Cfg,
    logger::logger_sender::LoggerSender,
    peer::{bt_peer::BtPeer, peer_session::PeerSession},
    torrent_parser::torrent::Torrent,
    tracker::tracker_handler::{TrackerHandler, TrackerHandlerError},
};
use std::{sync::Arc, thread};

/// Struct for handling the torrent download.
///
/// To create a new `TorrentHandler`, use TorrentHandler::new(torrent, config, logger_sender).
#[derive(Debug)]
pub struct TorrentHandler {
    torrent: Torrent,
    config: Cfg,
    logger_sender: LoggerSender,
    torrent_status: Arc<AtomicTorrentStatus>,
}

/// Posible torrent handler errors.
#[derive(Debug)]
pub enum TorrentHandlerError {
    TrackerError(TrackerHandlerError),
    TorrentStatusError(AtomicTorrentStatusError),
    StartingServerError(BtServerError),
}

impl TorrentHandler {
    /// Creates a new `TorrentHandler` from a torrent, a config and a logger sender.
    pub fn new(torrent: Torrent, config: Cfg, logger_sender: LoggerSender) -> Self {
        Self {
            torrent_status: Arc::new(AtomicTorrentStatus::new(&torrent, config.clone())),
            torrent,
            config,
            logger_sender,
        }
    }

    /// Starts the torrent download.
    ///
    /// First it connects to the tracker and gets the peers. Then it connects to each peer and starts the download.
    ///
    /// # Errors
    ///
    /// - `TrackerErr` if there was a problem connecting to the tracker or getting the peers.
    /// - `TorrentStatusError` if there was a problem using the `Torrent Status`.
    pub fn handle(&mut self) -> Result<(), TorrentHandlerError> {
        self.start_server()?;

        let tracker_handler =
            TrackerHandler::new(self.torrent.clone(), self.config.tcp_port.into())
                .map_err(TorrentHandlerError::TrackerError)?;
        self.logger_sender.info("Connected to tracker.");

        while !self.torrent_status.is_finished() {
            let peer_list = self.get_peers_list(&tracker_handler)?;
            self.logger_sender.info("Tracker peer list obteined.");

            // Iniciar conección con cada peer
            for peer in peer_list {
                let mut current_peers = self.torrent_status.current_peers();
                let mut is_finished = self.torrent_status.is_finished();

                // Si se llego  máximo de peers simultaneos esperar hasta que se libere uno
                while (current_peers >= self.config.max_peers_per_torrent as usize) && !is_finished
                {
                    thread::yield_now();

                    current_peers = self.torrent_status.current_peers();
                    is_finished = self.torrent_status.is_finished();
                }
                self.connect_to_peer(peer);
            }
        }
        self.logger_sender.info("Torrent download finished.");
        Ok(())
    }

    fn start_server(&mut self) -> Result<(), TorrentHandlerError> {
        let mut server = BtServer::new(
            self.torrent.clone(),
            self.config.clone(),
            self.torrent_status.clone(),
            self.logger_sender.clone(),
        );

        let builder =
            thread::Builder::new().name(format!("Server for Torrent: {}", self.torrent.info.name));
        let server_logger_sender = self.logger_sender.clone();

        let join = builder.spawn(move || match server.init() {
            Ok(_) => (),
            Err(err) => {
                server_logger_sender.error(&format!("The server couldn't be started: {:?}", err));
            }
        });
        match join {
            Ok(_) => (),
            Err(err) => self.logger_sender.error(&format!("{:?}", err)),
        }
        Ok(())
    }

    fn get_peers_list(
        &self,
        tracker_handler: &TrackerHandler,
    ) -> Result<Vec<BtPeer>, TorrentHandlerError> {
        let tracker_response = tracker_handler
            .get_peers_list()
            .map_err(TorrentHandlerError::TrackerError)?;
        Ok(tracker_response.peers)
    }

    fn connect_to_peer(&mut self, peer: BtPeer) {
        self.torrent_status.peer_connected();

        let peer_name = format!("{}:{}", peer.ip, peer.port);

        let mut peer_session = PeerSession::new(
            peer,
            self.torrent.clone(),
            self.torrent_status.clone(),
            self.config.clone(),
            self.logger_sender.clone(),
        );

        let builder = thread::Builder::new().name(format!(
            "Torrent: {} / Peer: {}",
            self.torrent.info.name, peer_name
        ));
        let peer_logger_sender = self.logger_sender.clone();

        let join = builder.spawn(move || match peer_session.start_outgoing_seeder() {
            Ok(_) => (),
            Err(err) => {
                peer_logger_sender.warn(&format!("{:?}", err));
            }
        });
        match join {
            Ok(_) => (),
            Err(err) => self.logger_sender.error(&format!("{:?}", err)),
        }
    }
}
