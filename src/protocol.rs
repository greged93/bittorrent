use crate::handshake::HandShake;
use crate::torrent::Torrent;
use miette::miette;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MY_PEER_ID: [u8; 20] = *b"00112233445566778899";

/// The bit torrent protocol stream. Wraps the tcp connection
/// and adds methods to handle the various message.
pub struct BitTorrentStream(tokio::net::TcpStream);

impl BitTorrentStream {
    /// Returns a new [`BitTorrentStream`].
    pub async fn new(address: String) -> Self {
        BitTorrentStream(
            tokio::net::TcpStream::connect(address)
                .await
                .expect("failed to connect to peer"),
        )
    }

    /// Handshakes with the peer for the provided torrent.
    pub async fn handshake(&mut self, torrent: Torrent) -> miette::Result<()> {
        let mut handshake = HandShake::new(torrent.raw_info_hash().as_ref(), MY_PEER_ID);
        let handshake = &mut handshake as *mut HandShake as *mut [u8; size_of::<HandShake>()];
        let handshake: &mut [u8; size_of::<HandShake>()] = unsafe { &mut *handshake };

        self.0
            .write_all(handshake)
            .await
            .expect("failed to write in stream");
        self.0
            .read_exact(handshake)
            .await
            .expect("failed to read stream");

        let offset = size_of::<HandShake>() - 20;
        println!("Peer ID: {}", hex::encode(&handshake[offset..]));

        Ok(())
    }

    /// Waits until a message with the provided id comes from the stream.
    pub async fn wait_message(&mut self, id: u8) -> miette::Result<()> {
        loop {
            let buffer = &mut [];
            self.0.read(buffer).await.map_err(|err| miette!(err))?;

            if let Some(i) = buffer.get(4) {
                if i == &id {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Send a message on the protocol, with the provided id and payload.
    pub async fn send_message(&mut self, id: u8, mut payload: Vec<u8>) -> miette::Result<()> {
        let length = (payload.len() + 5) as u32;

        let mut buffer = Vec::with_capacity(length as usize);
        buffer.extend(length.to_be_bytes());
        buffer.push(id);
        buffer.append(&mut payload);

        self.0
            .write_all(&buffer)
            .await
            .map_err(|err| miette!(err))?;

        Ok(())
    }
}
