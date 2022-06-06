use bit_torrent_rustico::bt_client::btclient::BtClient;
use bit_torrent_rustico::bt_client::btclient_error::BtClientError;
use bit_torrent_rustico::bt_client::error_message::ErrorMessage;
use core::time;
use std::env;
use std::thread::sleep;

fn main() -> Result<(), BtClientError> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 2 {
        return Err(BtClientError::ArgumentError(ErrorMessage::new(
            "Incorrect number of arguments. Only a directory path containing one or more torrents should be passed".to_string(),
        )));
    };
    let client = BtClient::init(arguments[1].clone());
    sleep(time::Duration::from_millis(200));
    let client = client?;

    client.run();

    Ok(())
}
