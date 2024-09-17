mod decode;
mod handshake;
mod peers;
mod protocol;
mod torrent;

use crate::peers::Peers;
use crate::protocol::{BitTorrentStream, SIXTEEN_KILO_BYTES};
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
    DownloadPiece {
        #[clap(short)]
        output: Option<PathBuf>,
        input: PathBuf,
        index: u32,
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
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");
            println!("{}", torrent)
        }
        Command::Peers { path } => {
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");
            let peers = Peers::get_peers(&torrent)
                .await
                .expect("failed to get peers");
            println!("{}", peers);
        }
        Command::Handshake { path, peer_address } => {
            let torrent = Torrent::read_from_file(path).expect("failed to read torrent");
            let mut stream = BitTorrentStream::new(&peer_address).await;
            stream.handshake(&torrent).await.unwrap();
        }
        Command::DownloadPiece {
            input,
            index,
            output,
        } => {
            // Get peers
            let torrent = Torrent::read_from_file(input).expect("failed to read torrent");
            let peers = Peers::get_peers(&torrent)
                .await
                .expect("failed to get peers");
            let peer = peers.0.first().expect("no peers");

            // Perform handshake
            let mut stream = BitTorrentStream::new(peer).await;
            stream.handshake(&torrent).await.expect("handshake failed");

            // Wait for the bitfield
            stream
                .wait_message(5)
                .await
                .expect("missing bitfield message");

            // Send an interested message
            stream
                .send_message(2, vec![])
                .await
                .expect("failed to send interested");

            // Wait for an unchoke message
            stream
                .wait_message(1)
                .await
                .expect("failed to get unchoke message");

            // Request each full piece
            let mut file = Vec::with_capacity(torrent.info.piece_length as usize);
            for (i, _) in (0..torrent.info.piece_length / SIXTEEN_KILO_BYTES).enumerate() {
                stream
                    .request_piece(
                        index,
                        (i as u32) * SIXTEEN_KILO_BYTES,
                        SIXTEEN_KILO_BYTES,
                        &mut file,
                    )
                    .await
                    .expect("failed to request piece");
            }

            // Request the last piece
            let size = torrent.info.piece_length % SIXTEEN_KILO_BYTES;
            let offset = torrent.info.piece_length - size;
            stream
                .request_piece(index, offset, size, &mut file)
                .await
                .expect("failed to request piece");

            if let Some(path) = output {
                std::fs::write(&path, file).expect("failed to write file");
                println!("Piece {index} downloaded to {path:?}");
            }
        }
    }
}
