use flume::{Receiver, Sender};
use picolimbo_proto::{nbt::Blob, Identifier};
use uuid::Uuid;

use crate::proto::{
    play::{Play, PlayLogin, PluginMessageOut},
    IntoPacket, Packet,
};

pub struct LimboPlayer {
    packets_tx: Sender<Packet>,
    packets_rx: Receiver<Packet>,
    uuid: Uuid,
    eid: i32,
}

impl LimboPlayer {
    pub fn new(
        uuid: Uuid,
        eid: i32,
        packets_tx: Sender<Packet>,
        packets_rx: Receiver<Packet>,
    ) -> Self {
        Self {
            packets_tx,
            packets_rx,
            uuid,
            eid,
        }
    }

    pub async fn send<P: IntoPacket>(&self, pkt: P) -> anyhow::Result<()> {
        let pkt = pkt.into_packet();
        self.packets_tx.send_async(pkt).await?;
        Ok(())
    }

    pub async fn recv(&self) -> anyhow::Result<Play> {
        self.packets_rx
            .recv_async()
            .await
            .map_err(anyhow::Error::from)
            .map(|it| match it {
                Packet::Play(play) => play,
                _ => unreachable!(),
            })
    }

    pub async fn handle_self(self) -> anyhow::Result<()> {
        // We have entered player stage
        self.send(PlayLogin {
            eid: self.eid,
            is_hardcore: true,
            gamemode: crate::proto::play::Gamemode::Adventure,
            prev_gamemode: crate::proto::play::Gamemode::Undefined,
            dimensions: vec![],
            registry_codec: Blob::new(),
            spawn_dimension: Identifier::from("minecraft:null"),
            dimension_name: Identifier::from("minecraft:null"),
            hashed_seed: 0x000000,
            max_players: 1,
            view_distance: 2,
            simulation_distance: 2,
            reduced_debug_info: false,
            enable_respawn_screen: false,
            is_debug: false,
            is_flat: false,
            has_death_pos: false,
        })
        .await?;
        self.send(PluginMessageOut {
            channel: Identifier::from("minecraft:brand"),
            data: vec![],
        })
        .await?;

        Ok(())
    }
}
