use super::constants::LOGGER_THREAD_NAME;
use super::logger_error::LoggerError;
use super::logger_sender::LoggerSender;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use std::fs;
use std::fs::File;
use std::io::Write;

use chrono::prelude::*;

/// A logger to log into a file
///
/// The logger works with channels. It has one channel to receive the information
/// and as many channels to send it. It can be used with multiple threads at the same time.
///
/// To clone the sender's channel it has a new_sender() method which returns a LoggerSender struct.
#[derive(Debug)]
pub struct Logger {
    sender: LoggerSender,
}

impl Logger {
    /// Constructs a new Logger to log
    ///
    /// In case of success it returns a Logger struct and creates a new log file at the directory path.
    ///
    /// It returns an LoggerError if:
    /// - A new file could not be created at the directory path given
    /// - There was a problem creating a new thread for the logger receiver
    pub fn new(dir_path: &str) -> Result<Self, LoggerError> {
        let (sender, receiver): (Sender<String>, Receiver<String>) = channel();

        let file = Self::create_log_file(dir_path)?;
        Self::spawn_log_receiver(receiver, file)?;

        Ok(Self {
            sender: LoggerSender::new(sender),
        })
    }

    /// Creates a new LoggerSender for the current Logger
    pub fn new_sender(&self) -> LoggerSender {
        self.sender.clone()
    }

    fn spawn_log_receiver(receiver: Receiver<String>, file: File) -> Result<(), LoggerError> {
        let builder = thread::Builder::new().name(LOGGER_THREAD_NAME.to_string());
        let result = builder.spawn(move || {
            let mut file = file;

            while let Ok(msg) = receiver.recv() {
                let msg: String = msg;
                let time = Local::now();
                let formated =
                    format!("{} {}\n", time.format("[%Y/%m/%d %H:%M:%S]"), msg).into_bytes();

                match file.write_all(&formated) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Error({err}) writing to the log"),
                }
            }
        });
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(LoggerError::SpawnThreadError),
        }
    }

    fn create_log_file(dir_path: &str) -> Result<File, LoggerError> {
        let time = Local::now();

        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(format!(
                "{}/{}.log",
                dir_path,
                time.format("%Y-%m-%d_%H-%M-%S")
            ));

        match file {
            Ok(file) => Ok(file),
            Err(_) => Err(LoggerError::BadLogPathError(dir_path.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::thread::sleep;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_good_log() {
        let path = "./test_good_log";
        let loggin = "log_test".to_string();
        let log_type = "info".to_string();
        create_log_and_assert_loggin(path, loggin, log_type);
    }

    #[test]
    fn test_info_log() {
        let path = "./test_info_log";
        let loggin = "[INFO]";
        let log_type = "info".to_string();
        create_log_and_assert_loggin(path, loggin.to_string(), log_type);
    }

    #[test]
    fn test_warn_log() {
        let path = "./test_warn_log";
        let loggin = "[WARN]";
        let log_type = "warn".to_string();
        create_log_and_assert_loggin(path, loggin.to_string(), log_type);
    }

    #[test]
    fn test_error_log() {
        let path = "./test_error_log";
        let loggin = "[ERROR]";
        let log_type = "error".to_string();
        create_log_and_assert_loggin(path, loggin.to_string(), log_type);
    }

    #[test]
    fn test_bad_path() {
        let path = "bad_path";

        let logger = Logger::new(path);

        assert!(logger.is_err());
    }

    #[test]
    fn test_multiple_loggin() {
        let path = "./test_multiple_loggin";
        let loggin = ["log_test_1", "log_test_2", "log_test_3"];
        fs::create_dir(path).unwrap();

        let logger = Logger::new(path).unwrap();

        let logger_sender_1 = logger.new_sender();
        let logger_sender_2 = logger.new_sender();
        let logger_sender_3 = logger.new_sender();

        thread::spawn(move || logger_sender_1.info(loggin[0]).unwrap());
        sleep(Duration::from_millis(100));
        thread::spawn(move || logger_sender_2.info(loggin[1]).unwrap());
        sleep(Duration::from_millis(100));
        thread::spawn(move || logger_sender_3.info(loggin[2]).unwrap());

        let paths = fs::read_dir(path).unwrap();
        for log_path in paths {
            let log = File::open(log_path.unwrap().path()).unwrap();
            let reader = BufReader::new(log);

            let mut counter = 0;
            for line in reader.lines() {
                let current_line = line.unwrap();

                assert!(current_line.contains(loggin[counter]));
                counter += 1;
            }
        }

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_multiple_loggin_same_thread() {
        let path = "./test_multiple_loggin_same_thread";
        let loggin = ["log_test_1", "log_test_2", "log_test_3"];
        fs::create_dir(path).unwrap();

        let logger = Logger::new(path).unwrap();

        let logger_sender = logger.new_sender();

        logger_sender.info(loggin[0]).unwrap();
        logger_sender.info(loggin[1]).unwrap();
        logger_sender.info(loggin[2]).unwrap();

        let paths = fs::read_dir(path).unwrap();
        for log_path in paths {
            let log = File::open(log_path.unwrap().path()).unwrap();
            let reader = BufReader::new(log);

            let mut counter = 0;
            for line in reader.lines() {
                let current_line = line.unwrap();

                assert!(current_line.contains(loggin[counter]));
                counter += 1;
            }
        }

        fs::remove_dir_all(path).unwrap();
    }

    // Auxiliary functions

    fn create_log_and_assert_loggin(path: &str, loggin: String, log_type: String) {
        fs::create_dir(path).unwrap();

        let logger = Logger::new(path).unwrap();
        let logger_sender = logger.new_sender();

        let loggin_assert = loggin.clone();

        thread::spawn(move || match log_type.as_str() {
            "info" => logger_sender.info(loggin.as_str()).unwrap(),
            "warn" => logger_sender.warn(loggin.as_str()).unwrap(),
            "error" => logger_sender.error(loggin.as_str()).unwrap(),
            _ => panic!("Unknown log type"),
        });

        let paths = fs::read_dir(path).unwrap();
        for log_path in paths {
            let log = File::open(log_path.unwrap().path()).unwrap();
            let reader = BufReader::new(log);

            for line in reader.lines() {
                let current_line = line.unwrap();

                assert!(current_line.contains(loggin_assert.as_str()));
            }
        }

        fs::remove_dir_all(path).unwrap();
    }
}
