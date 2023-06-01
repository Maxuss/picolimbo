use futures_lite::FutureExt;
use lobsterchat::lobster;
use picolimbo_proto::Protocol;

use crate::{
    client::ClientStream,
    player::LimboPlayer,
    proto::{
        handshake::{Handshake, PingResponse, ServerStatus, ServerVersion, Status, StatusResponse},
        login::{Login, LoginDisconnect, LoginSuccess},
        IntoPacket, Packet,
    },
};

pub async fn do_initial_handle(mut stream: ClientStream) -> anyhow::Result<()> {
    let Handshake::HandshakeInitial(hs) = stream.read::<Handshake>().await?;
    stream.reinject_protocol(Protocol::from_idx(hs.protocol_version)); // reinjecting protocol version

    match hs.next_state {
        crate::proto::handshake::HsNextState::Status => {
            let _status_request = stream.read::<Status>().await?;
            let response = StatusResponse {
                    status: ServerStatus {
                        description: lobster(format!("<#8802cc>This is Picolimbo, and you are using protocol <#efb217><italic><bold>{}", hs.protocol_version)),
                        version: ServerVersion {
                            name: "latest",
                            protocol: hs.protocol_version,
                        },
                        ..Default::default()
                    },
                };
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
                let uuid = uuid::Uuid::new_v4();
                let username = start.username;

                stream
                    .send(Packet::Login(Login::LoginSuccess(LoginSuccess {
                        username,
                        uuid,
                    })))
                    .await?;

                let player = LimboPlayer::new(
                    uuid,
                    stream.outgoing_packets(),
                    stream.inbound_packets(),
                    Protocol::from_idx(hs.protocol_version),
                );

                let stream_task = tokio::task::spawn(async move { stream.start().await });
                let player_task = tokio::task::spawn(async move { player.handle_self().await });

                let res = stream_task.race(player_task).await?;

                if let Err(e) = res {
                    tracing::debug!("Error during handling: {e}")
                }
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
