use miette::miette;
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

pub struct Torrent {
    announce: String,
    info: Info,
}

pub struct Info {
    length: usize,
    name: String,
    piece_length: usize,
    pieces: String,
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
        let pieces = as_str(info, "pieces")?;

        Ok(Self {
            announce,
            info: Info {
                name,
                pieces,
                piece_length,
                length,
            },
        })
    }
}

impl Display for Torrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tracker URL: {}\nLength: {}",
            self.announce, self.info.length
        )
    }
}
