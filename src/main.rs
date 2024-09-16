mod decode;
mod handshake;
mod peers;
mod protocol;
mod torrent;

use crate::peers::Peers;
use crate::protocol::BitTorrentStream;
use crate::torrent::Torrent;
use clap::{Parser, Subcommand};
use decode::Decoder;
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
            let peers = Peers::get_peers(torrent)
                .await
                .expect("failed to get peers");
            println!("{}", peers);
        }
        Command::Handshake { path, peer_address } => {
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");
            let mut stream = BitTorrentStream::new(peer_address).await;
            stream.handshake(torrent).await.unwrap();
        }
    }
}
