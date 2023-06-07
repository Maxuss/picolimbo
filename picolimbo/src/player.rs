use std::time::Duration;

use flume::{Receiver, Sender};
use lobsterchat::component::{AsComponent, Component};

use picolimbo_proto::{Identifier, Protocol};

use uuid::Uuid;

use crate::{
    config::{LimboJoinAction, PluginMessageData},
    proto::{
        play::{
            ChatMessage, ChatMessagePosition, Gamemode, KeepAliveClientbound, Play, PlayLogin,
            PlayerAbilities, PlayerInfo, PlayerPositionRotation, PluginMessageOut, SpawnPosition,
        },
        IntoPacket, Packet,
    },
    server::LimboServer,
};
use crate::proto::play::{ShowBossbar, TitleMessage, TitleSubtitle, TitleTimes};

pub struct LimboPlayer {
    packets_tx: Sender<Packet>,
    packets_rx: Receiver<Packet>,
    uuid: Uuid,
    ver: Protocol,
    server: LimboServer,
}

impl LimboPlayer {
    pub fn new(
        uuid: Uuid,
        packets_tx: Sender<Packet>,
        packets_rx: Receiver<Packet>,
        ver: Protocol,
        server: LimboServer,
    ) -> Self {
        Self {
            packets_tx,
            packets_rx,
            uuid,
            ver,
            server,
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

    #[async_recursion::async_recursion]
    async fn handle_join_action(&self, action: &LimboJoinAction) -> anyhow::Result<()> {
        match action {
            LimboJoinAction::SendMessage { send_message } => {
                self.send(ChatMessage {
                    message: send_message.clone(),
                    position: ChatMessagePosition::Chat,
                    sender: Uuid::new_v4(),
                })
                .await?;
            }
            LimboJoinAction::SendPluginMessage {
                send_plugin_message: PluginMessageData { channel, message },
            } => {
                self.send(PluginMessageOut {
                    channel: channel.clone(),
                    data: message.clone(),
                })
                .await?;
            }
            LimboJoinAction::SendActionBar { send_action_bar } => {
                self.send(ChatMessage {
                    message: send_action_bar.clone(),
                    position: ChatMessagePosition::ActionBar,
                    sender: Uuid::new_v4(),
                })
                .await?;
            }
            LimboJoinAction::MapForVersions { match_version } => {
                for (version, action) in match_version {
                    if *version == self.ver {
                        self.handle_join_action(action).await?;
                        break;
                    }
                }
            }
            LimboJoinAction::SendTitle { send_title } if self.ver >= Protocol::V1_8 => {
                let fade_in = send_title.fade_in.unwrap_or(20);
                let fade_out = send_title.fade_out.unwrap_or(20);
                let stay = send_title.stay.unwrap_or(100);
                self.send(TitleTimes {
                    fade_in,
                    stay,
                    fade_out,
                }).await?;
                if let Some(subtitle) = &send_title.subtitle {
                    let title = send_title.title.clone().unwrap_or(Component::text(String::new()));
                    self.send(TitleMessage {
                        message: title
                    }).await?;
                    self.send(TitleSubtitle {
                        message: subtitle.as_component()
                    }).await?;
                } else if let Some(title) = &send_title.title {
                    let title = title.as_component();
                    self.send(TitleMessage {
                        message: title
                    }).await?;
                }
            }
            LimboJoinAction::SendBossbar { send_bossbar } if self.ver >= Protocol::V1_9  => {
                self.send(ShowBossbar {
                    bossbar: send_bossbar.clone()
                }).await?;
            }
            _ => todo!()
        }
        Ok(())
    }

    pub async fn handle_self(self) -> anyhow::Result<()> {
        // We have entered the `play` stage

        self.send(PlayLogin {
            eid: 0,
            is_hardcore: true,
            gamemode: Gamemode::Survival,
            spawn_dimension: Identifier(
                "minecraft".to_owned(),
                self.server.config().dimension.clone(),
            ),
            hashed_seed: 0x0000000,
            max_players: 1,
            view_distance: 2,
            simulation_distance: 2,
            reduced_debug_info: false,
            enable_respawn_screen: false,
            is_debug: false,
            is_flat: true,
            has_death_pos: false,
        })
        .await?;

        self.send(PlayerAbilities {
            flags: 0x02,
            flying_speed: 0.,
            fov_mod: 0.1,
        })
        .await?;

        if self.ver < Protocol::V1_9 {
            self.send(PlayerPositionRotation {
                x: 0.,
                y: 64.,
                z: 0.,
                yaw: 0.,
                pitch: 0.,
                on_ground: false,
            }) // does not support high Y values
            .await?;
        } else {
            self.send(PlayerPositionRotation {
                x: 0.,
                y: 400.,
                z: 0.,
                yaw: 0.,
                pitch: 0.,
                on_ground: false,
            })
            .await?;
        }

        if self.ver >= Protocol::V1_19_3 {
            self.send(SpawnPosition {
                x: 0,
                y: 400,
                z: 0,
                rotation: 0.,
            })
            .await?;
        }

        if self.ver == Protocol::V1_16_4 {
            self.send(PlayerInfo {
                username: "A Limbo Player".to_string(),
                gamemode: 1,
                uuid: self.uuid,
            })
            .await?;
        }

        if self.ver >= Protocol::V1_13 {
            // self.send(SendCommands {}).await?;
            self.send(PluginMessageOut {
                channel: "minecraft:brand".to_owned(),
                data: self.server.config().server_brand.clone(),
            })
            .await?;
        } else {
            self.send(PluginMessageOut {
                channel: "MC|Brand".to_owned(),
                data: self.server.config().server_brand.clone(),
            })
            .await?;
        }

        for action in &self.server.config().on_join_actions {
            self.handle_join_action(action).await?;
        }

        let mut interval = tokio::time::interval(Duration::from_secs(3)); // sending keepalive every 3 seconds

        let packets_tx = self.packets_tx;

        let ka_tx_task = tokio::task::spawn(async move {
            loop {
                interval.tick().await;
                if (packets_tx
                    .send_async(
                        KeepAliveClientbound {
                            ka_id: rand::random(),
                        }
                        .into_packet(),
                    )
                    .await)
                    .is_err()
                {
                    break;
                }
            }
            self.server.remove_player();
            drop(self.server);

            drop(packets_tx);
        });

        ka_tx_task.await?;

        drop(self.packets_rx);

        Ok(())
    }
}
