use crate::torrent_handler::status::AtomicTorrentStatus;
use core::time;
use gtk::glib;
use std::{sync::Arc, thread::sleep};

#[derive(Debug)]
pub struct Statistics {
    pub torrent_name: String,
    pub info_hash: String,
    pub length: u32,
    pub pieces_amount: u32,
    pub peers_amount: usize,
    //completed: f32,
    pub downloaded_pieces_amount: usize,
    //active_connections: i32,
    // peers: Vec<BtPeer>
    // download_speed: i32,
    // upload_speed: i32,
}

impl Statistics {
    pub fn for_torrent(torrent_status: &Arc<AtomicTorrentStatus>) -> Self {
        let torrent = torrent_status.torrent.clone(); //TODO: no romper encap
        Self {
            torrent_name: torrent.name(),
            info_hash: torrent.info_hash(),
            length: torrent.length(),
            pieces_amount: torrent.total_pieces(),
            peers_amount: torrent_status.current_peers(),
            downloaded_pieces_amount: torrent_status.downloaded_pieces(),
        }
        //     completed: (),
        //     downloaded_pieces_amount: (),
        //     active_connections: (),
        //     peers: (),
        //     download_speed: (),
        //     upload_speed: ()
        // }
    }

    pub fn download_percentage(&self) -> f32 {
        self.downloaded_pieces_amount as f32 / self.pieces_amount as f32
    }

    pub fn torrent_name(&self) -> &str {
        &self.torrent_name
    }
}

pub struct Runner {
    torrent_status_list: Vec<Arc<AtomicTorrentStatus>>,
    sender: glib::Sender<Vec<Statistics>>,
}

#[derive(Debug)]
pub enum RunnerError {
    SenderError,
}

impl Runner {
    pub fn new(
        torrent_status_list: Vec<Arc<AtomicTorrentStatus>>,
        sender: glib::Sender<Vec<Statistics>>,
    ) -> Runner {
        Self {
            torrent_status_list,
            sender,
        }
    }

    pub fn run(&self) -> Result<(), RunnerError> {
        loop {
            self.sender
                .send(self.torrent_statistics())
                .map_err(|_err| RunnerError::SenderError)?;
            sleep(time::Duration::from_millis(500));
        }
    }

    pub fn torrent_statistics(&self) -> Vec<Statistics> {
        let mut statistics = Vec::new();
        for torrent_status in &self.torrent_status_list {
            statistics.push(Statistics::for_torrent(torrent_status));
        }

        statistics
    }
}
