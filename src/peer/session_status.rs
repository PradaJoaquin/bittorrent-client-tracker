use super::peer_message::Bitfield;

/// Represents our status in the peer session.
#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub choked: bool,
    pub interested: bool,
    pub bitfield: Bitfield,
    pub download_speed: f64,
    pub upload_speed: f64,
}

impl SessionStatus {
    pub fn new(bitfield: Bitfield) -> Self {
        Self {
            choked: true,
            interested: false,
            bitfield,
            download_speed: 0.0,
            upload_speed: 0.0,
        }
    }
}
