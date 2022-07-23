use std::net::TcpStream;

pub struct Request {
    pub stream: TcpStream,
}

impl Request {
    pub fn new(stream: TcpStream) -> Request {
        Request { stream: stream }
    }

    pub fn handle(&self) {

        // let body = self.stream.read_to_end().unwrap();

        // let http_request = Http::parse(&body);

        // match http_request.endpoint {
        //     "announce" => {self.handle_announce(http_request);},
        //     "stats" => {self.handle_stats(http_request);},
        //     _ => { Err(()) }
        // };
    }

    // fn handle_announce(&self) {
    //     let announce_response = AnnounceResponse::from(http_request.params);
    // }

    // fn handle_stats(&self) {
    //     let stats_response = Stats::from(http_request.params);
    //     self.send_response(response);
    // }

    // fn send_response(&self, response: String) {
    //    stream
    //    .write_all(create_response(buffer)?.as_bytes())
    //    .unwrap();
    //    stream.flush().unwrap();
    //  }
}
