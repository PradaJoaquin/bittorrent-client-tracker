use super::peer_status::PeerStatus;

/// Struct that represents a peer.
///
/// ## Fields
/// * `id`: The id of the peer.
/// * `ip`: The ip of the peer.
/// * `port`: The port of the peer.
/// * `status`: The current status of the peer.
/// * `key`: The key to use to differentiate between other peers *(Optional)*.
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: [u8; 20],
    pub ip: String,
    pub port: u16,
    pub status: PeerStatus,
    pub key: Option<String>, //link a wiki.theory.org:  https://bit.ly/3aTXQ3u
}
impl Peer {
    /// Creates a new peer.
    pub fn new(
        id: [u8; 20],
        ip: String,
        port: u16,
        key: Option<String>,
        status: PeerStatus,
    ) -> Peer {
        Peer {
            id,
            ip,
            port,
            status,
            key,
        }
    }
}
