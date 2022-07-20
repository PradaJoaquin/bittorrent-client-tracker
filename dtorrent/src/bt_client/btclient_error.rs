use crate::bt_client::error_message::ErrorMessage;
use logger::logger_error::LoggerError;

/// Represents an error that happened while initializing a BtClient struct
#[derive(Debug)]
pub enum BtClientError {
    ConfigurationFileError(ErrorMessage),
    TorrentDirectoryError(ErrorMessage),
    LogError(LoggerError),
    ArgumentError(ErrorMessage),
    UIBuildingError(ErrorMessage),
}

impl From<LoggerError> for BtClientError {
    fn from(err: LoggerError) -> BtClientError {
        BtClientError::LogError(err)
    }
}
