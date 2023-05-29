use lobsterchat::component::Component;
use serde::Serialize;
use uuid::Uuid;

use crate::{build_packets, varint_enum};

varint_enum!(in HsNextState {
    Status = 0x01,
    Login = 0x02
});

build_packets! { Handshake:
    packet HandshakeInitial(in 0x00) {
        protocol_version: i32 as varint,
        server_address: String,
        server_port: u16,
        next_state: HsNextState
    }
}

build_packets! { Status:
    packet StatusRequest(in 0x00) {
        // Empty packet
    };

    packet StatusResponse(out 0x00) {
        status: ServerStatus as json
    };

    packet PingRequest(in 0x01) {
        payload: i64
    };

    packet PingResponse(out 0x01) {
        payload: i64
    };
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerStatus {
    pub version: ServerVersion,
    pub players: ServerPlayers,
    pub description: Component,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            version: ServerVersion {
                name: "latest",
                protocol: 756,
            },
            players: ServerPlayers {
                max: 1,
                online: 1,
                sample: Default::default(),
            },
            description: Default::default(),
            favicon: Default::default(),
            enforces_secure_chat: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<ServerPlayerSingle>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerPlayerSingle {
    pub name: &'static str,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerVersion {
    pub name: &'static str,
    pub protocol: i32,
}
