use crate::peer::{bt_peer::BtPeer, session_status::SessionStatus};

#[derive(Debug, Clone)]
pub struct PeerStats {
    pub port: i64,
    pub ip: String,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub choked: bool,            // We are choked
    pub interested: bool,        // We are interested
    pub client_choked: bool,     // The other peer is choked by us
    pub client_interested: bool, // The other peer is interested in us
    pub peer_id: String,
}

impl PeerStats {
    pub fn for_peer(peer: BtPeer, session_status: SessionStatus) -> Self {
        Self {
            port: peer.port,
            ip: peer.ip,
            download_speed: session_status.download_speed,
            upload_speed: session_status.upload_speed,
            choked: session_status.choked,
            interested: session_status.interested,
            client_choked: session_status.peer_choked,
            client_interested: session_status.peer_interested,
            peer_id: Self::format_peer_id(&peer.peer_id),
        }
    }

    fn format_peer_id(peer_id: &Option<Vec<u8>>) -> String {
        match peer_id {
            Some(p) => format!("{:?}", String::from_utf8_lossy(p.as_slice())),
            None => "".to_string(),
        }
    }
}
