use crate::handshake::HandShake;
use crate::torrent::Torrent;
use miette::miette;
use std::mem::size_of;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MY_PEER_ID: [u8; 20] = *b"00112233445566778899";
pub const SIXTEEN_KILO_BYTES: u32 = 1 << 14;

/// The bit torrent protocol stream. Wraps the tcp connection
/// and adds methods to handle the various message.
pub struct BitTorrentStream(tokio::net::TcpStream);

impl BitTorrentStream {
    /// Returns a new [`BitTorrentStream`].
    pub async fn new(address: &str) -> Self {
        BitTorrentStream(
            tokio::net::TcpStream::connect(address)
                .await
                .expect("failed to connect to peer"),
        )
    }

    /// Connect to the tcp stream and request the torrent piece for the
    /// provided index.
    pub async fn connect_and_request_piece(
        address: &str,
        torrent: &Torrent,
        index: u32,
    ) -> miette::Result<Vec<u8>> {
        // Perform handshake
        let mut stream = BitTorrentStream::new(&address).await;
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
        let full = torrent.info.length / torrent.info.piece_length;

        // The piece length will be torrent.info.piece_length if the index of the
        // piece isn't the last one, or torrent.info.length % torrent.info.piece_length
        let piece_len = if index == full {
            torrent.info.length % torrent.info.piece_length
        } else {
            torrent.info.piece_length
        };

        // Request all full pieces
        for i in 0..piece_len / SIXTEEN_KILO_BYTES {
            stream
                .request_piece(index, i * SIXTEEN_KILO_BYTES, SIXTEEN_KILO_BYTES, &mut file)
                .await
                .expect("failed to request piece");
        }

        // Request the last piece
        let size = piece_len % SIXTEEN_KILO_BYTES;
        let offset = piece_len - size;
        stream
            .request_piece(index, offset, size, &mut file)
            .await
            .expect("failed to request piece");

        Ok(file)
    }

    /// Handshakes with the peer for the provided torrent.
    pub async fn handshake(&mut self, torrent: &Torrent) -> miette::Result<()> {
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

    /// Makes a request for a piece to the stream. Modifies the provided mutable
    /// reference to the file, appending the received payload to it.
    pub async fn request_piece(
        &mut self,
        index: u32,
        offset: u32,
        size: u32,
        file: &mut Vec<u8>,
    ) -> miette::Result<()> {
        // Build the payload: index, begin, length
        let mut payload = Vec::with_capacity(24);
        payload.extend(index.to_be_bytes());
        payload.extend(offset.to_be_bytes());
        payload.extend(size.to_be_bytes());

        // Send request
        self.send_message(6, payload).await?;

        // Wait for response
        let mut payload = self.wait_message(7).await?;

        // Split the payload: first 4 bytes are index, following 4 bytes are begin
        payload.drain(..8);
        file.append(&mut payload);

        Ok(())
    }

    /// Waits until a message with the provided id comes from the stream.
    pub async fn wait_message(&mut self, id: u8) -> miette::Result<Vec<u8>> {
        // Read the message length in bytes
        let mut length = [0u8; 4];
        self.0
            .read_exact(&mut length)
            .await
            .map_err(|err| miette!(err))?;
        let length = u32::from_be_bytes(length);

        // This is a heartbeat, return
        if length == 0 {
            return Ok(vec![]);
        }

        // Read the msg id
        let mut msg_id = [0u8; 1];
        self.0
            .read_exact(&mut msg_id)
            .await
            .map_err(|err| miette!(err))?;
        let msg_id = u8::from_be_bytes(msg_id);

        // Check the id is the expected one
        if msg_id != id {
            return Err(miette!("expected {id}, got {msg_id}"));
        }

        let mut payload = vec![0; (length - 1) as usize];
        self.0
            .read_exact(&mut payload)
            .await
            .map_err(|err| miette!(err))?;

        Ok(payload)
    }

    /// Send a message on the protocol, with the provided id and payload.
    pub async fn send_message(&mut self, id: u8, mut payload: Vec<u8>) -> miette::Result<()> {
        // The length of the message is the id + the payload length
        let length = (payload.len() + 1) as u32;

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
