use std::{net::TcpListener, sync::Arc};

use logger::logger_sender::LoggerSender;

use crate::http_server::request_handler::RequestHandler;
use crate::{
    http_server::thread_pool::pool::ThreadPool,
    tracker_status::atomic_tracker_status::AtomicTrackerStatus,
};

/// Struct that represents the current status of the tracker.
///
/// ## Fields
/// * `listener`: The TCP server binded to the socket, responsible of listening for connections.
/// * `pool`: A thread pool that provides worker threads, in order to favor parallel execution.
/// * `status`: Current status of the tracker.
/// * `logger_sender`: To log using the Logger.
pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
    status: Arc<AtomicTrackerStatus>,
    logger_sender: LoggerSender,
}

impl Server {
    /// Creates a new `Server`.
    pub fn init(
        status: Arc<AtomicTrackerStatus>,
        logger_sender: LoggerSender,
    ) -> std::io::Result<Server> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        Ok(Server {
            listener,
            pool: ThreadPool::new(4, logger_sender.clone()),
            status,
            logger_sender,
        })
    }

    /// Handles new connections to the server
    pub fn serve(&self) -> std::io::Result<()> {
        self.logger_sender.info("Serving on http://127.0.0.1:8080");

        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let mut request_handler = RequestHandler::new(stream);
            let logger = self.logger_sender.clone();
            self.pool.execute(move || {
                if let Err(error) = request_handler.handle() {
                    logger.error(&format!(
                        "An error occurred while attempting to handle a request: {:?}",
                        error
                    ));
                }
            });
        }
        Ok(())
    }
}
