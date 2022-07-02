use crate::bt_client::btclient::BtClient;
use crate::bt_client::btclient_error::BtClientError;
use crate::bt_client::statistics::Statistics;
use gtk::prelude::BuilderExtManual;
use gtk::{glib, Window};
use gtk::{prelude::*, ListStore};
use std::thread;

struct BitTorrentWindow {}
pub enum BitTorrentWindowError {
    WidgetBuildingError,
}

pub fn build_ui(app: &gtk::Application) -> Result<(), BitTorrentWindowError> {
    let glade_src = include_str!("test_ui.xml");
    let builder = gtk::Builder::from_string(glade_src);

    let window: Option<Window> = builder.object("window");
    match window {
        Some(window) => {
            window.set_application(Some(app));

            let mut bittorrent_window = BitTorrentWindow {};
            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            if let Err(error) = bittorrent_window.build(&builder, receiver) {
                return Err(error);
            };
            if bittorrent_window.start(sender).is_err() {
                return Err(BitTorrentWindowError::WidgetBuildingError);
            };

            window.show_all();

            Ok(())
        }
        None => Err(BitTorrentWindowError::WidgetBuildingError),
    }
}

impl BitTorrentWindow {
    pub fn build(
        &self,
        builder: &gtk::Builder,
        receiver: glib::Receiver<Vec<Statistics>>,
    ) -> Result<(), BitTorrentWindowError> {
        let liststore: Option<ListStore> = builder.object("liststore1");
        match liststore {
            Some(liststore) => {
                receiver.attach(None, move |statistics: Vec<Statistics>| {
                    for (index, stat) in statistics.iter().enumerate() {
                        let iter = match liststore.iter_from_string(index.to_string().as_str()) {
                            Some(iter) => iter,
                            None => liststore.append(),
                        };

                        liststore.set(
                            &iter,
                            &[
                                (0u32, &stat.torrent_name),
                                (1u32, &(stat.download_percentage() * 100_f32)),
                                (2u32, &stat.info_hash),
                                (3u32, &stat.length),
                                (4u32, &(stat.peers_amount as u32)), // TODO: should be connected peers!
                                (5u32, &stat.pieces_amount),
                                (6u32, &(stat.downloaded_pieces_amount as u32)),
                            ],
                        );
                    }

                    glib::Continue(true)
                });
                Ok(())
            }
            None => Err(BitTorrentWindowError::WidgetBuildingError),
        }
    }

    pub fn start(&mut self, sender: glib::Sender<Vec<Statistics>>) -> Result<(), BtClientError> {
        thread::spawn(move || {
            let client = BtClient::init("torrents".to_string());
            match client {
                Ok(client) => {
                    client.run(sender);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        });
        Ok(())
    }
}
