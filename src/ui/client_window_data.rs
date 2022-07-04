use super::setup::UserInterfaceError;
use crate::statistics::peer_stats::PeerStats;
use crate::statistics::torrent_stats::TorrentStats;
use gtk::{
    glib::{self, FormatSizeFlags},
    prelude::*,
    ListStore,
};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Mutex,
};

pub struct ClientWindowData {
    torrents_liststore: ListStore,
    peers_liststore: ListStore,
    last_torrents_statistics: Mutex<Vec<TorrentStats>>,
    selected_torrent_index: AtomicI32,
}

impl ClientWindowData {
    pub fn new(builder: &gtk::Builder) -> Result<Self, UserInterfaceError> {
        let torrents_liststore: ListStore = builder
            .object("torrents")
            .ok_or(UserInterfaceError::WindowDataError)?;
        let peers_liststore: ListStore = builder
            .object("peers")
            .ok_or(UserInterfaceError::WindowDataError)?;

        // Sort by torrent name
        //torrents_liststore.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);

        // Sort by peer download speed
        //peers_liststore.set_sort_column_id(gtk::SortColumn::Index(7), gtk::SortType::Descending);

        Ok(Self {
            last_torrents_statistics: Mutex::new(Vec::new()),
            torrents_liststore,
            peers_liststore,
            selected_torrent_index: AtomicI32::new(0),
        })
    }

    pub fn update_statistics(&self, statistics: Vec<TorrentStats>) {
        let mut torrent_stats = self.last_torrents_statistics.lock().unwrap();
        torrent_stats.clear();
        torrent_stats.extend(statistics);
    }

    pub fn update_torrent_liststore(&self) {
        let torrent_stats = self.last_torrents_statistics.lock().unwrap();
        for (index, statistics) in torrent_stats.iter().enumerate() {
            self.update_torrent_store_row(index, statistics);
        }
    }

    pub fn update_peer_liststore(&self) {
        let torrent_stats = self.last_torrents_statistics.lock().unwrap();
        self.peers_liststore.clear();
        for (peer_index, peer_stats) in torrent_stats[self.selected_torrent() as usize]
            .peers
            .iter()
            .enumerate()
        {
            self.update_peer_store_row(peer_index, peer_stats);
        }
    }

    fn selected_torrent(&self) -> i32 {
        self.selected_torrent_index.load(Ordering::Relaxed)
    }

    fn update_torrent_store_row(&self, row_num: usize, torrent_stats: &TorrentStats) {
        let tl_iter = match self
            .torrents_liststore
            .iter_from_string(row_num.to_string().as_str())
        {
            Some(iter) => iter,
            None => self.torrents_liststore.append(),
        };
        self.torrents_liststore.set(
            &tl_iter,
            &[
                (0u32, &torrent_stats.torrent_name),
                (1u32, &(torrent_stats.download_percentage() * 100_f32)),
                (2u32, &torrent_stats.info_hash),
                (
                    3u32,
                    &(glib::format_size_full(
                        torrent_stats.length as u64,
                        FormatSizeFlags::IEC_UNITS,
                    )),
                ),
                (4u32, &(torrent_stats.peers_amount as u32)),
                (5u32, &torrent_stats.pieces_amount),
                (6u32, &(torrent_stats.downloaded_pieces_amount as u32)),
                (7u32, &(torrent_stats.total_peers as u32)),
                (8u32, &self.format_speed(torrent_stats.download_speed)),
                (9u32, &self.format_speed(torrent_stats.upload_speed)),
                (10u32, &torrent_stats.eta),
            ],
        );
    }

    fn update_peer_store_row(&self, peer_index: usize, peer_stats: &PeerStats) {
        let pl_iter = match self
            .peers_liststore
            .iter_from_string(peer_index.to_string().as_str())
        {
            Some(iter) => iter,
            None => self.peers_liststore.append(),
        };
        self.peers_liststore.set(
            &pl_iter,
            &[
                (0u32, &peer_stats.ip),
                (1u32, &peer_stats.port),
                (2u32, &self.format_speed(peer_stats.download_speed)),
                (3u32, &self.format_speed(peer_stats.upload_speed)),
                (
                    4u32,
                    &self.format_state(peer_stats.choked, peer_stats.interested),
                ),
                (
                    5u32,
                    &self.format_state(peer_stats.client_choked, peer_stats.client_interested),
                ),
                (6u32, &peer_stats.peer_id),
                (7u32, &peer_stats.download_speed), // For sorting
                (8u32, &peer_stats.upload_speed),   // For sorting
            ],
        );
    }

    fn format_state(&self, choked: bool, interested: bool) -> String {
        let choked_str = if choked { "choked" } else { "unchoked" };
        let interested_str = if interested {
            "interested"
        } else {
            "not interested"
        };
        format!("{}/{}", choked_str, interested_str)
    }

    fn format_speed(&self, speed: f64) -> String {
        let speed_in_kilobytes = speed / 8_f64;
        if speed_in_kilobytes < 1024_f64 {
            format!("{:.2} KiB/s", speed_in_kilobytes)
        } else if speed_in_kilobytes < 1024_f64 * 1024_f64 {
            format!("{:.2} MiB/s", speed_in_kilobytes / 1024_f64)
        } else if speed_in_kilobytes < 1024_f64 * 1024_f64 * 1024_f64 {
            format!("{:.2} GiB/s", speed_in_kilobytes / 1024_f64 / 1024_f64)
        } else {
            format!(
                "{:.2} TiB/s",
                speed_in_kilobytes / 1024_f64 / 1024_f64 / 1024_f64
            )
        }
    }

    pub fn select_torrent(&self, new_index: i32) {
        self.selected_torrent_index
            .store(new_index, Ordering::Relaxed);
    }
}
