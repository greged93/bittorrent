use crate::decode::Decoder;
use itertools::Itertools;
use miette::miette;
use serde_json::{Map, Value};
use sha1::{Digest, Sha1};
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub struct Torrent {
    pub(crate) announce: String,
    pub(crate) info: Info,
}

impl Torrent {
    /// Reads the torrent from a file.
    pub fn read_from_file(path: PathBuf) -> miette::Result<Self> {
        let file_content = std::fs::read(path).map_err(|_| miette!("failed to read file"))?;
        let mut decoder = Decoder::new(&file_content);

        let value = decoder
            .decode()
            .map_err(|_| miette!("failed to decode file"))?;

        value.try_into()
    }

    /// Returns the info hash of the torrent.
    pub fn info_hash(&self) -> String {
        hex::encode(self.info.hash())
    }

    /// Returns the raw info hash of the torrent.
    pub fn raw_info_hash(&self) -> Vec<u8> {
        self.info.hash()
    }

    /// Returns the url encoded info hash.
    pub fn url_encoded_info_hash(&self) -> String {
        self.info_hash()
            .chars()
            .chunks(2)
            .into_iter()
            .map(|chunk| {
                chunk.fold(String::from("%"), |mut output, c| {
                    let _ = write!(output, "{c}");
                    output
                })
            })
            .collect()
    }
}

pub struct Info {
    pub(crate) length: u32,
    pub(crate) name: String,
    pub(crate) piece_length: u32,
    pub(crate) pieces_raw: Vec<u8>,
    pub(crate) pieces: String,
}

impl Info {
    /// Returns the sha-1 hash of the information.
    fn hash(&self) -> Vec<u8> {
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
            self.pieces_raw.len(),
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
        fn as_u32(object: &Map<String, Value>, key: &str) -> miette::Result<u32> {
            object
                .get(key)
                .ok_or(miette!("expected {key} field"))?
                .as_u64()
                .ok_or(miette!("expected usize"))
                .map(|x| x as u32)
        }

        let announce = as_str(object, "announce")?;

        let info = object
            .get("info")
            .ok_or(miette!("expected announce field"))?
            .as_object()
            .ok_or(miette!("expected object"))?;
        let length = as_u32(info, "length")?;
        let piece_length = as_u32(info, "piece length")?;
        let name = as_str(info, "name")?;
        // Reconstruct pieces from the hex string
        // by taking 2 chars and converting them to a byte
        let pieces = as_str(info, "pieces")?;
        let pieces_raw = hex::decode(&pieces).map_err(|_| miette!("decoding error"))?;

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
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}\nPiece Length: {}\nPiece Hashes:\n{}",
            self.announce,
            self.info.length,
            hex::encode(self.info.hash()),
            self.info.piece_length,
            pieces
        )
    }
}
