use std::{io::Cursor, time::Duration};

use anyhow::bail;
use picolimbo_proto::{BytesMut, Decodeable, Encodeable, ProtoError, Varint};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    time::timeout,
};

use crate::proto::{handshake::Handshake, IntoPacket, Packet};

pub struct ClientStream {
    inbound_packets_rx: flume::Receiver<Packet>,
    outgoing_packets_tx: flume::Sender<Packet>,
    writer: ClientStreamWriter,
    reader: ClientStreamReader,
}

impl ClientStream {
    pub fn new(tcp: TcpStream) -> Self {
        let (reader, writer) = tcp.into_split();
        let (outgoing_packets_tx, outgoing_packets_rx) = flume::bounded(16); // we are not sending much packets
        let (inbound_packets_tx, inbound_packets_rx) = flume::bounded(16); // we are not receiving much packets either

        let writer = ClientStreamWriter {
            writer,
            outgoing_packets_rx,
        };
        let reader = ClientStreamReader {
            reader,
            inbound_packets_tx,
        };

        Self {
            inbound_packets_rx,
            outgoing_packets_tx,
            writer,
            reader,
        }
    }

    pub fn inbound_packets(&self) -> flume::Receiver<Packet> {
        self.inbound_packets_rx.clone()
    }

    pub fn outgoing_packets(&self) -> flume::Sender<Packet> {
        self.outgoing_packets_tx.clone()
    }

    pub async fn read<D: Decodeable>(&mut self) -> anyhow::Result<D> {
        self.reader.read_packet().await
    }

    pub async fn send<E: Encodeable>(&mut self, enc: E) -> anyhow::Result<()> {
        self.writer.send(enc).await
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let write_task = tokio::task::spawn(self.writer.start());
        let read_task = tokio::task::spawn(self.reader.start());
        let (joined_a, joined_b) = tokio::join![write_task, read_task];
        joined_a??;
        joined_b??;
        Ok(())
    }
}

struct ClientStreamWriter {
    writer: OwnedWriteHalf,
    outgoing_packets_rx: flume::Receiver<Packet>,
}

impl ClientStreamWriter {
    async fn start(mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.outgoing_packets_rx.recv_async().await {
            let res = self.send(packet).await;
            if res.is_err() {
                // Connection dropped
                return Ok(());
            }
        }
        Ok(())
    }

    async fn send<E: Encodeable>(&mut self, enc: E) -> anyhow::Result<()> {
        let buffer_size = enc.predict_size();
        let mut buffer = BytesMut::with_capacity(buffer_size);
        enc.encode(&mut buffer)?;
        self.writer
            .write_all_buf(&mut buffer)
            .await
            .map_err(anyhow::Error::from)
    }
}

struct ClientStreamReader {
    reader: OwnedReadHalf,
    inbound_packets_tx: flume::Sender<Packet>,
}

impl ClientStreamReader {
    async fn start(mut self) -> anyhow::Result<()> {
        // empty packet sink
        loop {
            if let Ok(packet) = self.read_packet::<Handshake>().await {
                let res = self
                    .inbound_packets_tx
                    .send_async(packet.into_packet())
                    .await;

                if res.is_err() {
                    // connection dropped
                    return Ok(());
                }
            }
        }
    }

    async fn read_packet<D: Decodeable>(&mut self) -> anyhow::Result<D> {
        let packet_len = self.read_varint_async().await?.0;
        let mut buf = vec![0u8; packet_len as usize];
        let size_read = timeout(Duration::from_secs(5), self.reader.read(&mut buf)).await??;

        if size_read == 0 {
            bail!("Received 0 bytes from client")
        }

        let mut cursor = Cursor::new(&buf[..]);
        D::decode(&mut cursor).map_err(anyhow::Error::from)
    }

    async fn read_varint_async(&mut self) -> anyhow::Result<Varint> {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = self.reader.read_u8().await?;
            let value = i32::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7 * num_read).0;

            num_read += 1;

            if num_read > 5 {
                return Err(anyhow::anyhow!(ProtoError::VarintError(
                    "Varint is too large! Expected maximum 5 bytes long.",
                )));
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(Varint(result))
    }
}
