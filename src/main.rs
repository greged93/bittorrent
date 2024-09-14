mod decode;

use clap::{Parser, Subcommand};
use decode::Decoder;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Decode { input: String },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let command = Cli::parse();
    match command.command {
        Command::Decode { input } => {
            // Uncomment this block to pass the first stage
            let mut decoder = Decoder::new(&input);
            let value = decoder.decode().expect("expected value");
            println!("{}", value);
        }
    }
}
