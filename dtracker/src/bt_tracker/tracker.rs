use std::io;
use std::sync::Arc;

use logger::{logger_error::LoggerError, logger_receiver::Logger};

use crate::{
    http_server::server::Server, tracker_status::atomic_tracker_status::AtomicTrackerStatus,
};

/// Struct that represents the Tracker itself.
///
/// Serves as a starting point for the application.
pub struct BtTracker {
    _logger: Logger,
    server: Server,
}

#[derive(Debug)]
pub enum BtTrackerError {
    LoggerInitError(LoggerError),
    CreatingServerError(io::Error),
    StartingServerError(io::Error),
}

impl BtTracker {
    /// Creates a new BtTracker
    pub fn init() -> Result<Self, BtTrackerError> {
        let logger = Logger::new("./logs", 1000000).map_err(BtTrackerError::LoggerInitError)?; // TODO: Sacar de configs
        let logger_sender = logger.new_sender();

        let tracker_status = Arc::new(AtomicTrackerStatus::default());

        let server = Server::init(tracker_status.clone(), logger_sender)
            .map_err(BtTrackerError::CreatingServerError)?;

        Ok(Self {
            _logger: logger,
            server,
        })
    }

    /// Starts the server for handling requests.
    pub fn run(&self) -> Result<(), BtTrackerError> {
        self.server
            .serve()
            .map_err(BtTrackerError::StartingServerError)
    }
}
