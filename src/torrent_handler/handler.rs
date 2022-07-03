use super::status::{AtomicTorrentStatus, AtomicTorrentStatusError};
use crate::{
    config::cfg::Cfg,
    logger::logger_sender::LoggerSender,
    peer::{
        bt_peer::BtPeer,
        peer_session::{PeerSession, PeerSessionError},
    },
    torrent_parser::torrent::Torrent,
    tracker::tracker_handler::{TrackerHandler, TrackerHandlerError},
};
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

/// Struct for handling the torrent download.
///
/// To create a new `TorrentHandler`, use TorrentHandler::new(torrent, config, logger_sender).
#[derive(Debug)]
pub struct TorrentHandler {
    torrent: Torrent,
    config: Cfg,
    logger_sender: LoggerSender,
    torrent_status: Arc<AtomicTorrentStatus>,
    torrent_status_receiver: Receiver<usize>,
}

/// Posible torrent handler errors.
#[derive(Debug)]
pub enum TorrentHandlerError {
    TrackerError(TrackerHandlerError),
    TorrentStatusError(AtomicTorrentStatusError),
    PeerSessionError(PeerSessionError),
    TorrentStatusRecvError(mpsc::RecvError),
}

impl TorrentHandler {
    /// Creates a new `TorrentHandler` from a torrent, a config and a logger sender.
    pub fn new(torrent: Torrent, config: Cfg, logger_sender: LoggerSender) -> Self {
        let (torrent_status, torrent_status_receiver) =
            AtomicTorrentStatus::new(&torrent, config.clone());

        Self {
            torrent_status: Arc::new(torrent_status),
            torrent,
            config,
            logger_sender,
            torrent_status_receiver,
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
    /// - `TorrentStatusRecvError` if there was a problem receiving from the receiver of `Torrent Status`.
    pub fn handle(&mut self) -> Result<(), TorrentHandlerError> {
        let tracker_handler =
            TrackerHandler::new(self.torrent.clone(), self.config.tcp_port.into())
                .map_err(TorrentHandlerError::TrackerError)?;
        self.logger_sender.info("Connected to tracker.");

        while !self.torrent_status.is_finished() {
            let peer_list = self.get_peers_list(&tracker_handler)?;
            self.logger_sender.info("Tracker peer list obteined.");

            // Start connection with each peer
            for peer in peer_list {
                let current_peers = self.torrent_status.current_peers();

                // If we reached the maximum number of simultaneous peers, wait until the status tells us that one disconnected.
                if current_peers >= self.config.max_peers_per_torrent as usize {
                    // This while loop is done to prevent creating more peers than allowed when multiple peers are disconnected at the same time.
                    while self
                        .torrent_status_receiver
                        .recv()
                        .map_err(TorrentHandlerError::TorrentStatusRecvError)?
                        != self.config.max_peers_per_torrent as usize - 1
                    {
                        continue;
                    }
                }
                if self.torrent_status.is_finished() {
                    break;
                }
                self.connect_to_peer(peer)?;
            }
        }
        self.logger_sender.info("Torrent download finished.");
        Ok(())
    }

    /// Gets the status of the torrent.
    pub fn status(&self) -> Arc<AtomicTorrentStatus> {
        self.torrent_status.clone()
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

    fn connect_to_peer(&mut self, peer: BtPeer) -> Result<(), TorrentHandlerError> {
        self.torrent_status
            .peer_connected(&peer)
            .map_err(TorrentHandlerError::TorrentStatusError)?;
        let peer_name = format!("{}:{}", peer.ip, peer.port);

        let mut peer_session = PeerSession::new(
            peer,
            self.torrent.clone(),
            self.torrent_status.clone(),
            self.config.clone(),
            self.logger_sender.clone(),
        )
        .map_err(TorrentHandlerError::PeerSessionError)?;

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
        Ok(())
    }
}
