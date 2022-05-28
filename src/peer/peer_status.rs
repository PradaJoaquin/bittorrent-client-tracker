/// Represents our status in the peer session.
#[derive(Debug)]
pub struct PeerStatus {
    pub choked: bool,
    pub interested: bool,
}

impl Default for PeerStatus {
    fn default() -> Self {
        Self {
            choked: true,
            interested: false,
        }
    }
}
