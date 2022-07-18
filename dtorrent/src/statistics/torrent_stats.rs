use super::peer_stats::PeerStats;
use crate::torrent_handler::status::{AtomicTorrentStatus, AtomicTorrentStatusError};
use core::time;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TorrentStats {
    pub torrent_name: String,
    pub info_hash: String,
    pub length: u32,
    pub pieces_amount: u32,
    pub peers_amount: usize,
    pub downloaded_pieces_amount: usize,
    pub peers: Vec<PeerStats>,
    pub total_peers: usize,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub eta: String,
}

impl TorrentStats {
    pub fn for_torrent(
        torrent_status: &Arc<AtomicTorrentStatus>,
    ) -> Result<Self, AtomicTorrentStatusError> {
        let torrent = torrent_status.torrent.clone(); //TODO: no romper encap
        let mut peers = Vec::new();
        let peers_hashmap = torrent_status.get_connected_peers()?;
        for (peer, session) in peers_hashmap {
            peers.push(PeerStats::for_peer(peer, session));
        }

        let (seeders, leechers) = torrent_status.get_total_peers();
        let total_peers = seeders + leechers;

        Ok(Self {
            torrent_name: torrent.name(),
            info_hash: torrent.info_hash(),
            length: torrent.length(),
            pieces_amount: torrent.total_pieces(),
            peers_amount: torrent_status.current_peers(),
            downloaded_pieces_amount: torrent_status.downloaded_pieces(),
            peers,
            total_peers,
            download_speed: torrent_status.torrent_download_speed()?,
            upload_speed: torrent_status.torrent_upload_speed()?,
            eta: Self::format_eta(torrent_status)?,
        })
    }

    fn format_eta(
        torrent_status: &Arc<AtomicTorrentStatus>,
    ) -> Result<String, AtomicTorrentStatusError> {
        let down_speed = torrent_status.torrent_download_speed()? / 8_f64;
        let remaining_bytes =
            torrent_status.remaining_pieces() as u32 * torrent_status.torrent.piece_length();

        let remaining_kb = remaining_bytes / 1024;

        if down_speed == 0.0 {
            return Ok("-".to_string());
        }

        let eta = (remaining_kb as f64 / down_speed).ceil() as u64;
        let eta = time::Duration::from_secs(eta);
        let seconds = eta.as_secs() % 60;
        let minutes = (eta.as_secs() / 60) % 60;
        let hours = ((eta.as_secs() / 60) / 60) % 60;
        Ok(format!("{:#02}:{:#02}:{:#02}", hours, minutes, seconds))
    }

    pub fn download_percentage(&self) -> f32 {
        self.downloaded_pieces_amount as f32 / self.pieces_amount as f32
    }

    pub fn torrent_name(&self) -> &str {
        &self.torrent_name
    }
}
