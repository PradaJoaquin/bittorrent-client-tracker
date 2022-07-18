use super::torrent_stats::TorrentStats;
use crate::torrent_handler::status::{AtomicTorrentStatus, AtomicTorrentStatusError};
use core::time;
use gtk::glib;
use std::{sync::Arc, thread::sleep};

#[derive(Debug)]
pub enum StatisticsUpdaterError {
    SenderError,
    TorrentStatisticsError,
}
pub struct StatisticsUpdater {
    torrent_status_list: Vec<Arc<AtomicTorrentStatus>>,
    sender: glib::Sender<Vec<TorrentStats>>,
}

impl StatisticsUpdater {
    pub fn new(
        torrent_status_list: Vec<Arc<AtomicTorrentStatus>>,
        sender: glib::Sender<Vec<TorrentStats>>,
    ) -> StatisticsUpdater {
        Self {
            torrent_status_list,
            sender,
        }
    }

    pub fn run(&self) -> Result<(), StatisticsUpdaterError> {
        loop {
            self.sender
                .send(
                    self.torrent_statistics()
                        .map_err(|_| StatisticsUpdaterError::TorrentStatisticsError)?,
                )
                .map_err(|_| StatisticsUpdaterError::SenderError)?;

            sleep(time::Duration::from_millis(300)); //Only update the UI every 300ms
        }
    }

    pub fn torrent_statistics(&self) -> Result<Vec<TorrentStats>, AtomicTorrentStatusError> {
        let mut statistics = Vec::new();
        for torrent_status in &self.torrent_status_list {
            statistics.push(TorrentStats::for_torrent(torrent_status)?);
        }
        Ok(statistics)
    }
}
