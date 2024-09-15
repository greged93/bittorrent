use itertools::Itertools;
use miette::miette;
use serde_json::{Map, Value};
use sha1::{Digest, Sha1};
use std::fmt::{Display, Formatter};

pub struct Torrent {
    announce: String,
    info: Info,
}

pub struct Info {
    length: usize,
    name: String,
    piece_length: usize,
    pieces_raw: Vec<u8>,
    pieces: String,
}

impl Info {
    /// Returns the sha-1 hash of the information.
    pub fn hash(&self) -> Vec<u8> {
        let bytes = self.encode();
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        hasher.finalize().to_vec()
    }

    /// Encode the information by reconstructing it and converting it to
    /// a slice u8.
    fn encode(&self) -> Vec<u8> {
        let info = format!(
            "d6:lengthi{}e4:name{}:{}12:piece lengthi{}e6:pieces{}:",
            self.length,
            self.name.len(),
            self.name,
            self.piece_length,
            self.pieces.len(),
        );
        let mut encoded = info.as_bytes().to_vec();
        // extend the slice with the pieces
        encoded.extend(self.pieces_raw.clone());
        // extend the slice with the terminating e
        encoded.extend(b"e");
        encoded
    }
}

impl TryFrom<Value> for Torrent {
    type Error = miette::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let object = value.as_object().ok_or(miette!("expected object"))?;
        fn as_str(object: &Map<String, Value>, key: &str) -> miette::Result<String> {
            object
                .get(key)
                .ok_or(miette!("expected {key} field"))?
                .as_str()
                .ok_or(miette!("expected str"))
                .map(|x| x.to_string())
        }
        fn as_usize(object: &Map<String, Value>, key: &str) -> miette::Result<usize> {
            object
                .get(key)
                .ok_or(miette!("expected {key} field"))?
                .as_u64()
                .ok_or(miette!("expected usize"))
                .map(|x| x as usize)
        }

        let announce = as_str(object, "announce")?;

        let info = object
            .get("info")
            .ok_or(miette!("expected announce field"))?
            .as_object()
            .ok_or(miette!("expected object"))?;
        let length = as_usize(info, "length")?;
        let piece_length = as_usize(info, "piece length")?;
        let name = as_str(info, "name")?;
        // Reconstruct pieces from the hex string
        // by taking 2 chars and converting them to a byte
        let pieces = as_str(info, "pieces")?;
        let pieces_raw = pieces
            .as_bytes()
            .chunks(2)
            .filter_map(|x| {
                std::str::from_utf8(x)
                    .ok()
                    .and_then(|s| u8::from_str_radix(s, 16).ok())
            })
            .collect();

        Ok(Self {
            announce,
            info: Info {
                name,
                pieces_raw,
                pieces,
                piece_length,
                length,
            },
        })
    }
}

impl Display for Torrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pieces = self
            .info
            .pieces
            .chars()
            .chunks(40)
            .into_iter()
            .map(|x| x.collect::<String>().to_lowercase())
            .join("\n");
        write!(
            f,
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}\nPiece Length: {}\nPiece Hashes:\n{}\n\n",
            self.announce,
            self.info.length,
            hex::encode(self.info.hash()),
            self.info.piece_length,
            pieces
        )
    }
}
