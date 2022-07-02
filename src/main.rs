use bit_torrent_rustico::bt_client::btclient_error::BtClientError;
use bit_torrent_rustico::bt_client::error_message::ErrorMessage;
use bit_torrent_rustico::ui::setup;
use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::Application;
use std::env;

fn main() -> Result<(), BtClientError> {
    if env::args().count() != 2 {
        return Err(BtClientError::ArgumentError(ErrorMessage::new(
            "Incorrect number of arguments. Only a directory path containing one or more torrents should be passed".to_string(),
        )));
    };

    let app = Application::builder()
        .application_id("ar.uba.fi.la-deymoneta.bittorrent")
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_open(|app, _file, _some_str| {
        let _b = setup::build_ui(app);
    });

    app.run();
    Ok(())
}
