use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};

use bencoder::bencode::Bencode;

use crate::{
    announce::announce_response::AnnounceResponse,
    http::{http_method::HttpMethod, http_parser::Http, http_status::HttpStatus},
    stats::stats_updater::StatsUpdater,
    tracker_status::atomic_tracker_status::AtomicTrackerStatus,
};

/// Struct that represents a connection capable of listening to requests and returning an answer.
pub struct RequestHandler {
    pub stream: TcpStream,
}

#[derive(Debug)]
pub enum RequestHandlerError {
    InvalidEndpointError,
    ParseHttpError,
    GettingPeerIpError,
    FromUtfError(std::string::FromUtf8Error),
}

impl RequestHandler {
    /// Returns a new RequestHandler.
    ///
    /// ## Arguments
    /// * `stream`: a TcpStream responsible of reading HTTP requests and sending a response.
    pub fn new(stream: TcpStream) -> RequestHandler {
        RequestHandler { stream }
    }

    /// Handles a HTTP request and sends back a response in a successful scenario.
    /// Returns a RequestHandleError in the event of a request to an invalid endpoint.
    ///
    /// ## Arguments
    /// * `tracker_status`: The status of the tracker at the moment of handling the request.
    pub fn handle(
        &mut self,
        tracker_status: Arc<AtomicTrackerStatus>,
        stats_updater: Arc<StatsUpdater>,
    ) -> Result<(), RequestHandlerError> {
        // TODO: read HTTP message length correctly
        let mut buf = [0; 1024];
        self.stream.read(&mut buf).unwrap();

        // TODO: should match and send error (400 BAD REQUEST) through stream before returning error
        let http_request = Http::parse(&buf).map_err(|_| RequestHandlerError::ParseHttpError)?;

        let (status_line, response) = if http_request.method.eq(&HttpMethod::Get) {
            let response = match http_request.endpoint.as_str() {
                "/announce" => {
                    self.handle_announce(http_request, tracker_status, self.get_peer_ip()?)
                }
                "/stats" => self.handle_stats(http_request, tracker_status, stats_updater),
                _ => return Err(RequestHandlerError::InvalidEndpointError),
            };
            (HttpStatus::Ok, response)
        } else {
            (HttpStatus::NotFound, "".as_bytes().to_vec())
        };

        self.send_response(response, status_line).unwrap();

        Ok(())
    }

    fn handle_announce(
        &self,
        http_request: Http,
        tracker_status: Arc<AtomicTrackerStatus>,
        peer_ip: String,
    ) -> Vec<u8> {
        let response = AnnounceResponse::from(http_request.params, tracker_status, peer_ip);
        match response.failure_reason {
            Some(failure) => Bencode::encode(&failure),
            None => Bencode::encode(&response),
        }
    }

    /// Receives a `since` param that represents the period for statistics in hours.
    fn handle_stats(
        &self,
        http_request: Http,
        tracker_status: Arc<AtomicTrackerStatus>,
        stats_updater: Arc<StatsUpdater>,
    ) -> Vec<u8> {
        let since = http_request.params.get("since").unwrap();

        // Obtener cantidades de peers conectados, seeders, leechers y torrents a traves del stats_updater

        // Distribuir en "buckets" de a minutos / horas

        // Armar string JSON
        String::from("stats").as_bytes().to_vec()
    }

    fn create_response(mut contents: Vec<u8>, status_line: HttpStatus) -> Vec<u8> {
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
            status_line.to_string(),
            contents.len(),
        );
        let mut response = response.as_bytes().to_vec();
        response.append(&mut contents);
        response
    }

    fn send_response(&mut self, contents: Vec<u8>, status_line: HttpStatus) -> std::io::Result<()> {
        let response = Self::create_response(contents, status_line);

        self.stream.write_all(&response)?;
        self.stream.flush().unwrap();

        Ok(())
    }

    fn get_peer_ip(&self) -> Result<String, RequestHandlerError> {
        Ok(self
            .stream
            .peer_addr()
            .map_err(|_| RequestHandlerError::GettingPeerIpError)?
            .ip()
            .to_string())
    }
}
