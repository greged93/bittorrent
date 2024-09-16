/// The handshake data for the TCP connection
/// with the bit torrent protocol.
#[repr(C)]
pub struct HandShake {
    length: u8,
    protocol: [u8; 19],
    reserved: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl HandShake {
    /// Construct a [`HandShake`]
    pub fn new(info_hash: &[u8], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            protocol: *b"BitTorrent protocol",
            reserved: [0u8; 8],
            info_hash: info_hash.try_into().expect("failed to convert info hash"),
            peer_id,
        }
    }
}
