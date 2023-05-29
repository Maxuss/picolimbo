use picolimbo_proto::{BytesMut, Encodeable};
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::proto::Packet;

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
            let mut buffer = BytesMut::with_capacity(packet.predict_size()); // creating buffer with approximate size
            packet.encode(&mut buffer)?;
            self.writer.write_all_buf(&mut buffer).await?;
        }
        Ok(())
    }
}

struct ClientStreamReader {
    reader: OwnedReadHalf,
    inbound_packets_tx: flume::Sender<Packet>,
}

impl ClientStreamReader {
    async fn start(mut self) -> anyhow::Result<()> {
        // empty packet sink
        Ok(())
    }
}
