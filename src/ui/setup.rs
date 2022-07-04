use super::client_window::ClientWindow;
use crate::bt_client::btclient::BtClient;
use crate::bt_client::btclient_error::BtClientError;
use crate::statistics::torrent_stats::TorrentStats;
use gtk::glib;
use std::thread;

pub enum UserInterfaceError {
    WidgetBuildingError,
    ClientError(BtClientError),
    WindowDataError,
    WindowBuildingError,
}

pub fn start_dtorrent_application(
    app: &gtk::Application,
    torrents_directory: String,
) -> Result<(), UserInterfaceError> {
    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    start_btclient(sender, torrents_directory).map_err(UserInterfaceError::ClientError)?;

    let client_window = ClientWindow::new(receiver)?;
    client_window.update_on_click()?;
    client_window.display_on(app);

    Ok(())
}

pub fn start_btclient(
    sender: glib::Sender<Vec<TorrentStats>>,
    torrents_directory: String,
) -> Result<(), BtClientError> {
    thread::spawn(move || match BtClient::init(torrents_directory) {
        Ok(client) => client.run(sender),
        Err(btclient_error) => eprintln!("{:?}", btclient_error),
    });
    Ok(())
}
