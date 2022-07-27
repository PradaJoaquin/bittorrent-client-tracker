use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use chrono::{DateTime, Local};

use crate::{torrent_swarm::swarm::Swarm, tracker_peer::peer::Peer};

/// Struct that represents the current status of the tracker.
///
/// ## Fields
/// * `torrents`: The current torrents supported by the tracker. The key is the torrent `Info Hash`. The value is the `Torrent Status`.
/// * `last_updated`: The last time the tracker status was updated.
pub struct AtomicTrackerStatus {
    torrent_swarms: Mutex<HashMap<[u8; 20], Swarm>>,
    // [u8; 20] is the info hash of the torrent.
    last_updated: Mutex<DateTime<Local>>,
}

impl Default for AtomicTrackerStatus {
    /// Creates a new tracker status.
    fn default() -> Self {
        AtomicTrackerStatus {
            torrent_swarms: Mutex::new(HashMap::new()),
            last_updated: Mutex::new(Local::now()),
        }
    }
}

impl AtomicTrackerStatus {
    /// Adds or updates a peer for a torrent in the tracker status.
    pub fn incoming_peer(&self, info_hash: [u8; 20], peer: Peer) {
        let mut swarms = self.lock_swarms();
        let torrent_swarm = swarms.entry(info_hash).or_insert_with(Swarm::default);
        torrent_swarm.peers.push(peer);
        torrent_swarm.last_updated = Local::now();

        self.update_last_updated();

        // TODO: write in disk the new status of the tracker.
    }

    /// Gets the current torrents supported by the tracker and their peers.
    pub fn get_swarms(&self) -> HashMap<[u8; 20], Swarm> {
        self.lock_swarms().clone()
    }

    fn update_last_updated(&self) {
        *self.lock_last_updated() = Local::now();
    }

    fn lock_swarms(&self) -> MutexGuard<HashMap<[u8; 20], Swarm>> {
        self.torrent_swarms.lock().unwrap() // Unwrap is safe here because we're the only ones who call this function.
    }

    fn lock_last_updated(&self) -> MutexGuard<DateTime<Local>> {
        self.last_updated.lock().unwrap() // Unwrap is safe here because we're the only ones who call this function.
    }
}

#[cfg(test)]
mod tests {
    use crate::tracker_peer::peer_status::PeerStatus;

    use super::*;

    #[test]
    fn test_incoming_peer() {
        let status = AtomicTrackerStatus::default();
        let peer = create_test_peer();
        status.incoming_peer([0; 20], peer);
        assert_eq!(status.get_swarms().len(), 1);
    }

    fn create_test_peer() -> Peer {
        let peer_status = PeerStatus {
            uploaded: 0,
            downloaded: 0,
            left: 0,
            event: None,
            last_seen: Local::now(),
            real_ip: None,
        };

        Peer::new([0; 20], "0".to_string(), 0, None, peer_status)
    }
}
