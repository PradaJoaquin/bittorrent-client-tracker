use std::collections::HashMap;

use super::announce_request::AnnounceRequest;

/// Struct representing the response of a tracker announce request.
///
/// # Fields
/// * `failure_reason`:  If present, then no other keys may be present. The value is a human-readable error message as to why the request failed.
/// * `warning_message`: Similar to failure reason, but the response still gets processed normally. The warning message is shown just like an error.
/// * `interval`: Interval in seconds that the client should wait between sending regular requests to the tracker.
/// * `min_interval`: Minimum announce interval. If present clients must not reannounce more frequently than this.
/// * `tracker_id`: A string that the client should send back on its next announcements. If absent and a previous announce sent a tracker id, do not discard the old value; keep using it.
/// * `complete`: number of peers with the entire file, i.e. seeders.
/// * `incomplete`: number of non-seeder peers, aka "leechers".
/// * `peers`: (dictionary model) The value is a list of dictionaries, each with the following keys:
///    - **peer_id**: peer's self-selected ID, as described above for the tracker request (string)
///    - **ip**: peer's IP address either IPv6 (hexed) or IPv4 (dotted quad) or DNS name (string)
///    - **port**: peer's port number (integer)
/// * `peers_binary`: peers: (binary model) Instead of using the dictionary model described above, the peers value may be a string consisting of multiples of 6 bytes. First 4 bytes are the IP address and last 2 bytes are the port number. All in network (big endian) notation.
#[derive(Debug)]
pub struct AnnounceResponse {
    pub failure_reason: Option<String>,
    pub warning_message: Option<String>,
    pub interval: u32,
    pub min_interval: Option<u32>,
    pub tracker_id: Option<String>,
    pub complete: u32,
    pub incomplete: u32,
    // pub peers: Vec<Peer>,
    // pub peers_binary: Vec<u8>,
}

impl AnnounceResponse {
    /// Creates a new AnnounceResponse from a HashMap containing the query parameters of the announce request.
    pub fn from(query_params: HashMap<String, String>) -> Self {
        let announce_request = AnnounceRequest::new_from(query_params);

        let failure_reason = match announce_request {
            Ok(_) => None,
            Err(announce_request_error) => Some(announce_request_error.to_string()),
        };

        // TODO: Create peer, notify status of a new request, build response with list of peers.

        Self {
            failure_reason,
            warning_message: None,
            interval: 0,
            min_interval: None,
            tracker_id: None,
            complete: 0,
            incomplete: 0,
            // peers: Vec::new(),
            // peers_binary: Vec::new(),
        }
    }
}
