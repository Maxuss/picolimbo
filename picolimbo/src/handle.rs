use lobsterchat::lobster;

use crate::{
    client::ClientStream,
    proto::{
        handshake::{Handshake, PingResponse, ServerStatus, ServerVersion, Status, StatusResponse},
        login::LoginDisconnect,
        IntoPacket,
    },
};

pub async fn do_initial_handle(mut stream: ClientStream) -> anyhow::Result<()> {
    if let Handshake::HandshakeInitial(hs) = stream.read::<Handshake>().await? {
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
                stream.send(
                    LoginDisconnect {
                        reason: lobster("<#ba5df4>You have been disconnected for reason: <#f4e05d>Unimplemented"),
                    }.into_packet()
                ).await?;
            }
        }
    }
    Ok(())
}
