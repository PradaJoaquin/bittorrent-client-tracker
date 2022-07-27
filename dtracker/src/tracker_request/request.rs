use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    announce::announce_response::AnnounceResponse,
    http::{http_method::HttpMethod, http_parser::Http, http_status::HttpStatus},
};

pub struct Request {
    pub stream: TcpStream,
}

#[derive(Debug)]
pub enum RequestError {
    InvalidEndpointError,
    ParseHttpError,
}

impl Request {
    pub fn new(stream: TcpStream) -> Request {
        Request { stream }
    }

    pub fn handle(&mut self) -> Result<(), RequestError> {
        let mut buf = vec![];
        self.stream.read_to_end(&mut buf).unwrap();

        // TODO: should match and send error (400 BAD REQUEST) through stream before returning error
        let http_request = Http::parse(&buf).map_err(|_| RequestError::ParseHttpError)?;

        let (status_line, response) = if http_request.method.eq(&HttpMethod::Get) {
            let response = match http_request.endpoint.as_str() {
                "/announce" => self.handle_announce(http_request),
                "/stats" => self.handle_stats(http_request),
                _ => return Err(RequestError::InvalidEndpointError),
            };
            (HttpStatus::Ok, response)
        } else {
            (HttpStatus::NotFound, "".to_string())
        };

        self.send_response(response.as_str(), status_line).unwrap();

        Ok(())
    }

    fn handle_announce(&self, http_request: Http) -> String {
        let announce_response = AnnounceResponse::from(http_request.params);
        String::from("")
    }

    fn handle_stats(&self, http_request: Http) -> String {
        // let stats_response = Stats::from(http_request.params);
        String::from("")
    }

    fn create_response(contents: &str, status_line: HttpStatus) -> std::io::Result<String> {
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            status_line.to_string(),
            contents.len(),
            contents
        );

        Ok(response)
    }

    fn send_response(&mut self, contents: &str, status_line: HttpStatus) -> std::io::Result<()> {
        self.stream
            .write_all(Self::create_response(contents, status_line)?.as_bytes())
            .unwrap();
        self.stream.flush().unwrap();

        Ok(())
    }
}
