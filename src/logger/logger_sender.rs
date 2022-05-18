use super::logger_error::LoggerError;
use std::sync::mpsc::Sender;

/// A LoggerSender representing the sender channel connected to a Logger
#[derive(Debug)]
pub struct LoggerSender {
    sender_clone: Sender<String>,
}

impl LoggerSender {
    /// Creates a new LoggerSender from a clone of an existing sender.
    pub fn new(sender_clone: Sender<String>) -> Self {
        Self { sender_clone }
    }

    /// Sends the information to write to the log connected to the LoggerSender.
    ///
    /// It returns an error if:
    /// - Couldn't send the information to the receiver
    pub fn send(&self, value: &str) -> Result<(), LoggerError> {
        match self.sender_clone.send(value.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(LoggerError::SendError(value.to_string())),
        }
    }
}
