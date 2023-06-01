use std::{io::Cursor, time::Duration};

use crate::proto::{
    play::{KeepAliveServerbound, PacketMapping, Play},
    Packet,
};
use anyhow::bail;
use futures_lite::FutureExt;
use picolimbo_proto::{BytesMut, Decodeable, Encodeable, Protocol, Varint};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    time::timeout,
};

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

        // assuming latest version of protocol until client sends the required packet
        let writer = ClientStreamWriter {
            writer,
            outgoing_packets_rx,
            protocol: Protocol::latest(),
        };
        let reader = ClientStreamReader {
            reader,
            inbound_packets_tx,
            staging: [0; 512],
            codec: BufferingCodec::new(),
            protocol: Protocol::latest(),
        };

        Self {
            inbound_packets_rx,
            outgoing_packets_tx,
            writer,
            reader,
        }
    }

    pub fn reinject_protocol(&mut self, proto: Protocol) {
        self.reader.protocol = proto;
        self.reader.codec.proto = proto;
        self.writer.protocol = proto;
    }

    pub fn inbound_packets(&self) -> flume::Receiver<Packet> {
        self.inbound_packets_rx.clone()
    }

    pub fn outgoing_packets(&self) -> flume::Sender<Packet> {
        self.outgoing_packets_tx.clone()
    }

    pub async fn read<D: Decodeable + std::fmt::Debug>(&mut self) -> anyhow::Result<D> {
        self.reader.read_packet().await
    }

    pub async fn send<E: Encodeable>(&mut self, enc: E) -> anyhow::Result<()> {
        self.writer.send(enc).await
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let write_task = tokio::task::spawn(async move { self.writer.start().await });
        let read_task = tokio::task::spawn(async move { self.reader.start().await });

        let _ = write_task.race(read_task).await?;
        Ok(())
    }
}

struct ClientStreamWriter {
    writer: OwnedWriteHalf,
    outgoing_packets_rx: flume::Receiver<Packet>,
    protocol: Protocol,
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
        enc.encode(&mut buffer, self.protocol)?;
        self.writer
            .write_all_buf(&mut buffer)
            .await
            .map_err(anyhow::Error::from)
    }
}

struct ClientStreamReader {
    reader: OwnedReadHalf,
    inbound_packets_tx: flume::Sender<Packet>,
    staging: [u8; 512],
    codec: BufferingCodec,
    protocol: Protocol,
}

impl ClientStreamReader {
    async fn start(mut self) -> anyhow::Result<()> {
        // empty packet sink
        loop {
            let packet = self
                .await_read_specific_packet::<KeepAliveServerbound>(
                    KeepAliveServerbound::id_for_proto(self.protocol),
                )
                .await?; // we don't care about any packets except for keepalives
            let res = self
                .inbound_packets_tx
                .send_async(Packet::Play(Play::KeepAliveServerbound(packet)))
                .await;

            if res.is_err() {
                // connection dropped
                return Ok(());
            }
        }
    }

    async fn await_read_specific_packet<D: Decodeable>(
        &mut self,
        packet_id: i32,
    ) -> anyhow::Result<D> {
        loop {
            if let Some(packet) = self.codec.read_packet_or_consume::<D>(packet_id)? {
                return Ok(packet);
            }

            let timeout_duration: Duration = Duration::from_secs(5);
            let size_read =
                timeout(timeout_duration, self.reader.read(&mut self.staging)).await??;
            if size_read == 0 {
                bail!("Received 0 bytes from client")
            }

            let bytes = &self.staging[..size_read];
            self.codec.accept_bytes(bytes);
        }
    }

    async fn read_packet<D: Decodeable>(&mut self) -> anyhow::Result<D> {
        loop {
            if let Some(packet) = self.codec.try_read_next::<D>()? {
                return Ok(packet);
            }

            let timeout_duration: Duration = Duration::from_secs(5);
            let size_read =
                timeout(timeout_duration, self.reader.read(&mut self.staging)).await??;
            if size_read == 0 {
                bail!("Received 0 bytes from client")
            }

            let bytes = &self.staging[..size_read];
            self.codec.accept_bytes(bytes);
        }
    }
}

struct BufferingCodec {
    received_bytes: Vec<u8>,
    pub proto: Protocol,
}

impl BufferingCodec {
    pub fn new() -> Self {
        Self {
            received_bytes: Vec::with_capacity(512),
            proto: Protocol::latest(),
        }
    }

    pub fn accept_bytes(&mut self, bytes: &[u8]) {
        self.received_bytes.extend(bytes);
    }

    pub fn read_packet_or_consume<D: Decodeable>(&mut self, id: i32) -> anyhow::Result<Option<D>> {
        let mut cursor = Cursor::new(&self.received_bytes[..]);
        let packet_matching = if let Ok(length) = Varint::decode(&mut cursor, self.proto) {
            let lfl = cursor.position() as usize;

            if self.received_bytes.len() - lfl >= length.0 as usize {
                cursor = Cursor::new(&self.received_bytes[lfl..lfl + length.0 as usize]);

                let proto_id = Varint::decode(&mut cursor, self.proto)?.0;
                if proto_id == id {
                    // This is the correct packet, we can now read it
                    let packet = D::decode(&mut cursor, self.proto)?;

                    let bytes_read = length.0 as usize + lfl;
                    self.received_bytes = self.received_bytes.split_off(bytes_read);

                    Some(packet)
                } else {
                    // Consuming the packet
                    let bytes_read = length.0 as usize + lfl;
                    self.received_bytes = self.received_bytes.split_off(bytes_read);
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        Ok(packet_matching)
    }

    pub fn try_read_next<D: Decodeable>(&mut self) -> anyhow::Result<Option<D>> {
        let mut cursor = Cursor::new(&self.received_bytes[..]);
        let packet = if let Ok(length) = Varint::decode(&mut cursor, self.proto) {
            let lfl = cursor.position() as usize;

            if self.received_bytes.len() - lfl >= length.0 as usize {
                cursor = Cursor::new(&self.received_bytes[lfl..lfl + length.0 as usize]);

                let packet = D::decode(&mut cursor, self.proto)?;

                let bytes_read = length.0 as usize + lfl;
                self.received_bytes = self.received_bytes.split_off(bytes_read);

                Some(packet)
            } else {
                None
            }
        } else {
            None
        };
        Ok(packet)
    }
}
