use crate::decode::Decoder;
use crate::torrent::Torrent;
use miette::miette;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};

/// The peers in the network.
pub struct Peers(pub(crate) Vec<String>);

#[derive(Serialize)]
struct PeersQueryParams {
    peer_id: String,
    port: String,
    uploaded: u32,
    downloaded: u32,
    left: u32,
    compact: u8,
}

impl Peers {
    /// Get peers for the provided torrent.
    pub async fn get_peers(torrent: &Torrent) -> miette::Result<Self> {
        let params = PeersQueryParams {
            peer_id: "00112233445566778899".to_string(),
            port: "6881".to_string(),
            uploaded: 0,
            downloaded: 0,
            left: torrent.info.length,
            compact: 0,
        };
        let info_hash = torrent.url_encoded_info_hash();
        let encoded_params = serde_urlencoded::to_string(&params).map_err(|err| miette!(err))?;
        let encoded_params = format!("{}&info_hash={}", encoded_params, info_hash);

        let url = format!("{}?{}", torrent.announce, encoded_params);

        let res = reqwest::get(url).await.map_err(|err| miette!(err))?;
        let raw_res = res.bytes().await.map_err(|err| miette!(err))?;

        let mut decoder = Decoder::new(raw_res.as_ref());
        let res = decoder.decode()?;
        dbg!(&res);

        res.try_into()
    }
}

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
        let map = value.as_object().ok_or(miette!("expected obj"))?;

        let peers = map.get("peers").ok_or(miette!("missing peers key"))?;
        let peers = peers.as_array().ok_or(miette!("expected arr for peers"))?;

        let mut addresses = Vec::new();
        for peer in peers {
            let peer = peer.as_object().ok_or(miette!("expected obj for peer"))?;
            let ip = peer
                .get("ip")
                .and_then(|x| x.as_str())
                .ok_or(miette!("expected str for ip"))?;
            let port = peer
                .get("port")
                .and_then(|x| x.as_u64())
                .ok_or(miette!("expected str for port"))?;
            addresses.push(format!("{ip}:{port}"))
        }

        Ok(Peers(addresses))
    }
}
