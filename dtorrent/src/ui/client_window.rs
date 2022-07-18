use std::sync::Arc;

use super::client_window_data::ClientWindowData;
use super::setup::UserInterfaceError;
use crate::statistics::torrent_stats::TorrentStats;
use gtk::glib::Receiver;
use gtk::prelude::*;
use gtk::{glib, Window};

pub struct ClientWindow {
    window: Window,
    window_data: Arc<ClientWindowData>,
    builder: gtk::Builder,
}

impl ClientWindow {
    pub fn new(receiver: Receiver<Vec<TorrentStats>>) -> Result<Self, UserInterfaceError> {
        let glade_src = include_str!("test_ui.xml");
        let builder = gtk::Builder::from_string(glade_src);

        let window: Window = builder
            .object("window")
            .ok_or(UserInterfaceError::WindowBuildingError)?;
        window.set_title("dTorrent");
        window
            .set_icon_from_file("./dtorrent/src/ui/logo.ico")
            .unwrap();

        let window_data = Arc::new(ClientWindowData::new(&builder)?);
        let window_data_clone = window_data.clone();
        receiver.attach(None, move |statistics: Vec<TorrentStats>| {
            window_data_clone.update_statistics(statistics);
            window_data_clone.update_torrent_liststore();
            window_data_clone.update_peer_liststore();
            glib::Continue(true)
        });

        window.show_all();

        Ok(Self {
            builder,
            window,
            window_data,
        })
    }

    pub fn update_on_click(&self) -> Result<(), UserInterfaceError> {
        let treeview: gtk::TreeView = self
            .builder
            .object("torrent_treeview")
            .ok_or(UserInterfaceError::WindowBuildingError)?;
        treeview.set_activate_on_single_click(true);

        let window_data_clone = self.window_data.clone();
        treeview.connect_row_activated(move |_, row_path, _| {
            window_data_clone.select_torrent(row_path.indices()[0]);
            window_data_clone.update_peer_liststore();
        });

        Ok(())
    }

    pub fn display_on(&self, app: &gtk::Application) {
        self.window.set_application(Some(app));
    }
}
