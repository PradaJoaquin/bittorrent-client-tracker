use super::logger_error::LoggerError;
use std::sync::mpsc::Sender;
use std::thread;

/// A LoggerSender representing the sender channel connected to a Logger
///
/// There are three ways to write to the log:
///  - `info()` to log information.
///  - `warn()` to log a non critical warning.
///  - `error()` to log a critical error.
///
/// To clone the LoggerSender simply call the `clone()` method.
#[derive(Debug, Clone)]
pub struct LoggerSender {
    sender_clone: Sender<String>,
}

impl LoggerSender {
    /// Creates a new LoggerSender from a clone of an existing sender.
    pub fn new(sender_clone: Sender<String>) -> Self {
        Self { sender_clone }
    }

    /// Writes an Info type log to the connected logger
    ///
    /// It returns an error if:
    /// - Couldn't send the information to the receiver
    pub fn info(&self, value: &str) -> Result<(), LoggerError> {
        let formated_value = format!("[{}] [INFO] - {}", self.get_thread_name(), value);
        self.send(formated_value)
    }

    /// Writes a Warn type log to the connected logger
    ///
    /// It returns an error if:
    /// - Couldn't send the information to the receiver
    pub fn warn(&self, value: &str) -> Result<(), LoggerError> {
        let formated_value = format!("[{}] [WARN] - {}", self.get_thread_name(), value);
        self.send(formated_value)
    }

    /// Writes an Error type log to the connected logger
    ///
    /// It returns an error if:
    /// - Couldn't send the information to the receiver
    pub fn error(&self, value: &str) -> Result<(), LoggerError> {
        let formated_value = format!("[{}] [ERROR] - {}", self.get_thread_name(), value);
        self.send(formated_value)
    }

    fn send(&self, value: String) -> Result<(), LoggerError> {
        match self.sender_clone.send(value.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(LoggerError::SendError(value)),
        }
    }

    fn get_thread_name(&self) -> String {
        let current_thread = thread::current();
        match current_thread.name() {
            Some(name) => name.to_string(),
            None => "unnamed-thread".to_string(),
        }
    }
}
