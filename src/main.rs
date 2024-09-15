mod decode;
mod torrent;

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
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
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
    }
}
