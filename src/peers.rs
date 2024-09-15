use itertools::Itertools;
use miette::miette;
use serde_json::Value;
use std::fmt::{Display, Formatter};

/// The peers in the network.
pub struct Peers(Vec<String>);

impl Display for Peers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for p in &self.0 {
            write!(f, "{p}\n")?;
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
