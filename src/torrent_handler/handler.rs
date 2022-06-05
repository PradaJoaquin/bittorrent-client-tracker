use super::{
    constants,
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
}

impl TorrentHandler {
    /// Creates a new `TorrentHandler` from a torrent, a config and a logger sender.
    pub fn new(torrent: Torrent, config: Cfg, logger_sender: LoggerSender) -> Self {
        Self {
            torrent_status: Arc::new(AtomicTorrentStatus::new(&torrent)),
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
        // Inicializar Torrent Server con self.config.tcp_port

        // self.logger_sender.info("Servidor inicializado");

        let tracker_handler =
            TrackerHandler::new(self.torrent.clone(), self.config.tcp_port.into())
                .map_err(TorrentHandlerError::TrackerError)?;

        while !self.torrent_download_finished()? {
            let tracker_response = tracker_handler
                .get_peers_list()
                .map_err(TorrentHandlerError::TrackerError)?;

            // Iniciar conección con cada peer
            for peer in tracker_response.peers {
                let mut current_peers = self
                    .torrent_status
                    .current_peers()
                    .map_err(TorrentHandlerError::TorrentStatusError)?;

                let mut remaining_pieces = self
                    .torrent_status
                    .remaining_pieces()
                    .map_err(TorrentHandlerError::TorrentStatusError)?;

                let mut is_finished = self.torrent_download_finished()?;

                // Si se llego  máximo de peers simultaneos esperar hasta que se libere uno
                while (current_peers >= constants::MAX_CURRENT_PEERS
                    || current_peers >= remaining_pieces)
                    && !is_finished
                {
                    thread::yield_now();

                    current_peers = self
                        .torrent_status
                        .current_peers()
                        .map_err(TorrentHandlerError::TorrentStatusError)?;

                    remaining_pieces = self
                        .torrent_status
                        .remaining_pieces()
                        .map_err(TorrentHandlerError::TorrentStatusError)?;

                    is_finished = self.torrent_download_finished()?;
                }
                self.connect_to_peer(peer)?;
            }
        }

        println!("Torrent terminado!");

        Ok(())
    }

    fn torrent_download_finished(&mut self) -> Result<bool, TorrentHandlerError> {
        self.torrent_status
            .is_finished()
            .map_err(TorrentHandlerError::TorrentStatusError)
    }

    fn connect_to_peer(&mut self, peer: BtPeer) -> Result<(), TorrentHandlerError> {
        self.torrent_status
            .peer_connected()
            .map_err(TorrentHandlerError::TorrentStatusError)?;

        let peer_name = format!("{}:{}", peer.ip, peer.port);

        let peer_name_clone = peer_name.clone();

        let mut peer_session =
            PeerSession::new(peer, self.torrent.clone(), self.torrent_status.clone());

        let builder = thread::Builder::new().name(format!(
            "Torrent: {} / Peer: {}",
            self.torrent.info.name, peer_name
        ));
        let peer_logger_sender = self.logger_sender.clone();

        let join = builder.spawn(move || match peer_session.start() {
            Ok(_) => (),
            Err(err) => {
                peer_logger_sender
                    .error(&format!("Error: {:?}, with peer: {}", err, peer_name_clone));
            }
        });
        match join {
            Ok(_) => (),
            Err(err) => self
                .logger_sender
                .error(&format!("Error: {:?}, with peer: {}", err, peer_name)),
        }
        Ok(())
    }
}
