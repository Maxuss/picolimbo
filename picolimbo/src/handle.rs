use lobsterchat::lobster;

use crate::{
    client::ClientStream,
    proto::{
        handshake::{Handshake, PingResponse, ServerStatus, ServerVersion, Status, StatusResponse},
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
                    tracing::info!("Got ping request: {:?}", ping);
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
            crate::proto::handshake::HsNextState::Login => unimplemented!(),
        }
    }
    Ok(())
}
