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
    Decode {
        input: String,
    },
    Info {
        path: PathBuf,
    },
    Peers {
        path: PathBuf,
    },
    Handshake {
        path: PathBuf,
        peer_address: String,
    },
    #[clap(name = "download_piece")]
    DownloadPiece {
        #[clap(short)]
        output: Option<PathBuf>,
        input: PathBuf,
        index: u32,
    },
    Download {
        #[clap(short)]
        output: Option<PathBuf>,
        input: PathBuf,
    },
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
            let torrent = Torrent::read_from_file(&path).expect("failed to read torrent");
            println!("{}", torrent)
        }
        Command::Peers { path } => {
            let torrent = Torrent::read_from_file(&path).expect("failed to read torrent");
            let peers = Peers::get_peers(&torrent)
                .await
                .expect("failed to get peers");
            println!("{}", peers);
        }
        Command::Handshake { path, peer_address } => {
            let torrent = Torrent::read_from_file(&path).expect("failed to read torrent");
            let mut stream = BitTorrentStream::new(&peer_address).await;
            stream.handshake(&torrent).await.unwrap();
        }
        Command::DownloadPiece {
            input,
            index,
            output,
        } => {
            // Get peers
            let torrent = Torrent::read_from_file(&input).expect("failed to read torrent");
            let peers = Peers::get_peers(&torrent)
                .await
                .expect("failed to get peers");
            let peer = peers.0.first().expect("no peers");

            let file = BitTorrentStream::connect_and_request_piece(peer, &torrent, index)
                .await
                .expect("failed to get piece");

            if let Some(path) = output {
                std::fs::write(&path, file).expect("failed to write file");
                println!("Piece {index} downloaded to {path:?}");
            }
        }
        Command::Download { input, output } => {
            // Get peers
            let torrent = Torrent::read_from_file(&input).expect("failed to read torrent");
            let peers = Peers::get_peers(&torrent)
                .await
                .expect("failed to get peers");
            let full = torrent.info.length / torrent.info.piece_length;

            // Split the indexes in chunks of peers, otherwise you risk
            // hitting a "Peer connection reset" issue.
            let indexes: Vec<u32> = (0..=full).collect();
            let peers_len = peers.0.len();
            let indexes = indexes.chunks(peers_len).collect::<Vec<_>>();
            let mut file = Vec::with_capacity(torrent.info.length as usize);

            // Create an iterator of futures which poll all available peers
            // for the torrent file.
            for group in indexes {
                let futs = group
                    .iter()
                    .zip(peers.0.iter().cycle())
                    .map(|(index, peer)| {
                        BitTorrentStream::connect_and_request_piece(peer, &torrent, *index)
                    });
                let pieces = futures::future::join_all(futs)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<Vec<u8>>, _>>()
                    .expect("failed to collect pieces");

                pieces.into_iter().for_each(|mut p| file.append(&mut p));
            }

            if let Some(path) = output {
                std::fs::write(&path, file).expect("failed to write file");
                println!("Downloaded {input:?} to {path:?}.");
            }
        }
    }
}
