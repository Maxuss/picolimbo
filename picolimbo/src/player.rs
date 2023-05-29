use flume::{Receiver, Sender};

use crate::proto::Packet;

pub struct LimboPlayer {
    packets_tx: Sender<Packet>,
    packets_rx: Receiver<Packet>,
}

impl LimboPlayer {
    pub fn new(packets_tx: Sender<Packet>, packets_rx: Receiver<Packet>) -> Self {
        Self {
            packets_tx,
            packets_rx,
        }
    }
}
