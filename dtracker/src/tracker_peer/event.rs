/// Posible announce events that can be sent to the tracker.
///
/// ## Fields
/// * `started`: The peer has started downloading the torrent.
/// * `stopped`: The peer has stopped downloading the torrent.
/// * `completed`: The peer has completed downloading the torrent.
#[derive(Debug, Clone)]
pub enum PeerEvent {
    Started,
    Stopped,
    Completed,
}
