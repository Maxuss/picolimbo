use std::net::SocketAddr;

use futures_lite::FutureExt;
use lobsterchat::{
    component::{Colored, Component, NamedColor},
    lobster,
};
use picolimbo_proto::Protocol;

use crate::{
    client::ClientStream,
    player::LimboPlayer,
    proto::{
        handshake::{
            Handshake, PingResponse, ServerPlayers, ServerStatus, ServerVersion, Status,
            StatusResponse,
        },
        login::{Login, LoginDisconnect, LoginSuccess},
        IntoPacket, Packet,
    },
    server::LimboServer,
};

pub async fn handle_client(
    mut stream: ClientStream,
    addr: SocketAddr,
    server: LimboServer,
) -> anyhow::Result<()> {
    let Handshake::HandshakeInitial(hs) = stream.read::<Handshake>().await?;
    let protocol = Protocol::from_idx(hs.protocol_version);
    stream.reinject_protocol(protocol); // reinjecting protocol version

    match hs.next_state {
        crate::proto::handshake::HsNextState::Status => {
            let _status_request = stream.read::<Status>().await?;
            let ver_name = format!("{}-{}", Protocol::V1_7_2, Protocol::latest())
                .replace('V', "")
                .replace('_', ".");

            let response = StatusResponse {
                status: ServerStatus {
                    description: server.config().motd.to_owned(),
                    version: ServerVersion {
                        name: ver_name,
                        protocol: if protocol == Protocol::Legacy {
                            Protocol::Legacy as i32
                        } else {
                            protocol as i32
                        },
                    },
                    players: ServerPlayers {
                        max: server.config().max_players as i32,
                        online: server.online_players() as i32,
                        sample: vec![],
                    },
                    ..Default::default()
                },
            };

            drop(server); // dropping server reference, we don't need it anymore

            stream.send(response.into_packet()).await?;
            let ping = stream.read::<Status>().await?;
            if let Status::PingRequest(ping) = ping {
                stream
                    .send(
                        PingResponse {
                            payload: ping.payload,
                        }
                        .into_packet(),
                    )
                    .await?;
            }
        }
        crate::proto::handshake::HsNextState::Login => {
            // perform basic handling, then delegate it all to a `LimboPlayer`
            let login_start = stream.read::<Login>().await?;

            if let Login::LoginStart(start) = login_start {
                if !server.try_add_player() {
                    stream
                        .send(
                            LoginDisconnect {
                                reason: server.config().full_message.clone().unwrap_or_else(|| {
                                    Component::text("Disconnected: Server is full!")
                                        .color(NamedColor::Red)
                                }),
                            }
                            .into_packet(),
                        )
                        .await?;
                    return Ok(());
                }

                let uuid = uuid::Uuid::new_v4();
                let username = start.username;

                stream
                    .send(Packet::Login(Login::LoginSuccess(LoginSuccess {
                        username: username.clone(),
                        uuid,
                    })))
                    .await?;

                let player = LimboPlayer::new(
                    uuid,
                    stream.outgoing_packets(),
                    stream.inbound_packets(),
                    Protocol::from_idx(hs.protocol_version),
                    server,
                );

                let stream_task = tokio::task::spawn(async move { stream.start().await });
                let player_task = tokio::task::spawn(async move { player.handle_self().await });

                tracing::info!(
                    "Player {username} [{ip}/{protocol}] has joined the limbo",
                    ip = addr.ip()
                );

                stream_task.race(player_task).await??;

                tracing::info!("Player {username} disconnected");
            } else {
                stream
                        .send(
                            LoginDisconnect {
                                reason: lobster("<red>Invalid packet! Expected LoginStart but received <gold>{login_start:?}"),
                            }
                            .into_packet(),
                        )
                        .await?;
            }
        }
    }

    Ok(())
}
