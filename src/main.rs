mod decode;

use clap::{Parser, Subcommand};
use decode::decode;

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
            let value = decode(&mut input.as_str()).expect("expected value");
            println!("{}", value);
        }
    }
}
