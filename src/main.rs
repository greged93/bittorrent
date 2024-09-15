mod decode;
mod torrent;
mod utils;

use crate::torrent::Torrent;
use clap::{Parser, Subcommand};
use decode::Decoder;
use itertools::Itertools;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Decode { input: String },
    Info { path: PathBuf },
    Peers { path: PathBuf },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() {
    let command = Cli::parse();
    match command.command {
        Command::Decode { input } => {
            let mut decoder = Decoder::new(input.as_bytes());
            let value = decoder.decode().expect("expected value");
            println!("{}", value);
        }
        Command::Info { path } => {
            let file_content = std::fs::read(path).expect("failed to read file");
            let mut decoder = Decoder::new(&file_content);

            let value = decoder.decode().expect("expected value");
            let torrent: Torrent = value.try_into().expect("failed to convert value");
            println!("{}", torrent)
        }
        Command::Peers { path } => {
            let file_content = std::fs::read(path).expect("failed to read file");
            let mut decoder = Decoder::new(&file_content);

            let value = decoder.decode().expect("expected value");
            let torrent: Torrent = value.try_into().expect("failed to convert value");

            let params = [
                format!("info_hash={}", torrent.url_encoded_info_hash()),
                "peer_id=00112233445566778899".into(),
                "port=6881".into(),
                "uploaded=0".into(),
                "downloaded=0".into(),
                format!("left={}", torrent.info.length),
                "compact=1".into(),
            ]
            .into_iter()
            .join("&");
            let url = format!("{}?{}", torrent.announce, params);

            let res = reqwest::get(url).await.expect("failed to get peers");
            let raw_res = res.bytes().await.expect("missing text");
            println!("{:?}", raw_res);

            let mut decoder = Decoder::new(raw_res.as_ref());
            let res = decoder.decode().expect("failed to decode answer");
            println!("{}", res);
        }
    }
}
