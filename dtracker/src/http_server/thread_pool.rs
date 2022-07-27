use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use logger::logger_sender::LoggerSender;

use super::worker::{Message, Worker};

/// Struct that represents a thread pool that spawns a specified number of worker threads and allows to process connections concurrently.
/// Each idle thread in the pool is available to handle a task.
/// When a thread is done processing its task, it is returned to the pool of idle threads, ready to handle a new task.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
    logger_sender: LoggerSender,
}

impl ThreadPool {
    /// Creates a new ThreadPool with a given size.
    /// The size is the number of threads in the pool.
    /// If the size is zero or a negative number, the `new` function will panic.
    pub fn new(size: usize, logger_sender: LoggerSender) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                logger_sender.clone(),
            ));
        }

        ThreadPool {
            workers,
            sender,
            logger_sender,
        }
    }

    /// Receives a closure and assigns it to a thread in the pool to run.
    pub fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(closure);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.logger_sender
            .info("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        self.logger_sender.info("Shutting down all workers.");

        for worker in &mut self.workers {
            self.logger_sender
                .info(format!("Shutting down worker {}", worker.id).as_str());
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
