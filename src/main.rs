mod decode;
mod peers;
mod torrent;

use crate::peers::{HandShake, Peers};
use crate::torrent::Torrent;
use clap::{Parser, Subcommand};
use decode::Decoder;
use itertools::Itertools;
use std::mem::size_of;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    Handshake { path: PathBuf, peer_address: String },
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
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");
            println!("{}", torrent)
        }
        Command::Peers { path } => {
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");

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

            let mut decoder = Decoder::new(raw_res.as_ref());
            let res = decoder.decode().expect("failed to decode answer");
            let peers: Peers = res.try_into().expect("failed to convert value to peers");
            println!("{}", peers);
        }
        Command::Handshake { path, peer_address } => {
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");

            let mut stream = tokio::net::TcpStream::connect(peer_address)
                .await
                .expect("failed to connect to peer");

            let mut handshake =
                HandShake::new(torrent.raw_info_hash().as_ref(), *b"00112233445566778899");
            let handshake = &mut handshake as *mut HandShake as *mut [u8; size_of::<HandShake>()];
            let handshake: &mut [u8; size_of::<HandShake>()] = unsafe { &mut *handshake };

            stream
                .write_all(handshake)
                .await
                .expect("failed to write in stream");
            stream
                .read_exact(handshake)
                .await
                .expect("failed to read stream");

            let offset = size_of::<HandShake>() - 20;
            println!("Peer ID: {}", hex::encode(&handshake[offset..]));
        }
    }
}
