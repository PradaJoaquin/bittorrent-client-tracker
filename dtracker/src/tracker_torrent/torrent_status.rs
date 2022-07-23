use chrono::{DateTime, Local};

use crate::tracker_peer::peer::Peer;

/// Struct that represents the status of a torrent.
///
/// ## Fields
/// * `peers`: The current peers of the torrent.
/// * `last_updated`: The last time the torrent status was updated.
#[derive(Debug, Clone)]
pub struct TorrentStatus {
    pub peers: Vec<Peer>,
    pub last_updated: DateTime<Local>,
}

impl Default for TorrentStatus {
    /// Creates a new tracker status.
    fn default() -> Self {
        TorrentStatus {
            peers: Vec::new(),
            last_updated: Local::now(),
        }
    }
}
