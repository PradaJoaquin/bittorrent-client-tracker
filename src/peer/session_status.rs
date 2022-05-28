/// Represents our status in the peer session.
#[derive(Debug)]
pub struct SessionStatus {
    pub choked: bool,
    pub interested: bool,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self {
            choked: true,
            interested: false,
        }
    }
}
