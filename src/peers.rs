use itertools::Itertools;
use miette::miette;
use serde_json::Value;
use std::fmt::{Display, Formatter};

/// The peers in the network.
pub struct Peers(Vec<String>);

impl Display for Peers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for p in &self.0 {
            writeln!(f, "{p}")?;
        }
        Ok(())
    }
}

impl TryFrom<Value> for Peers {
    type Error = miette::Error;

    fn try_from(value: Value) -> miette::Result<Self> {
        let map = value.as_object().ok_or(miette!("expected object"))?;

        let peers = map.get("peers").ok_or(miette!("missing peers key"))?;
        let peers = peers.as_str().ok_or(miette!("expected str for peers"))?;
        let peers = hex::decode(peers).map_err(|_| miette!("failed decoding peers hex str"))?;

        let peers = peers
            .chunks(6)
            .filter_map(|peer| {
                let ip = &peer[..4].iter().map(|b| format!("{b}")).join(".");
                let port = format!(
                    "{}",
                    u32::from_str_radix(&hex::encode(&peer[4..]), 16).ok()?
                );
                Some(format!("{ip}:{port}"))
            })
            .collect::<Vec<_>>();
        Ok(Peers(peers))
    }
}

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
