/// Represents our status in the peer session.
#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub choked: bool,
    pub interested: bool,
    pub download_speed: f64,
    pub upload_speed: f64,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self {
            choked: true,
            interested: false,
            download_speed: 0.0,
            upload_speed: 0.0,
        }
    }
}
